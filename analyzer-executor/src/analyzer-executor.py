import base64
import hashlib
import json

from multiprocessing import Process, Pipe
from multiprocessing.connection import Connection
from typing import Any, Union, Optional, Tuple

import boto3

import graph_description_pb2


class Subgraph(object):
    def __init__(self, s: str):
        self.subgraph = graph_description_pb2.GraphDescription()
        self.subgraph.ParseFromString(s)


class ExecutionHit(object):
    def __init__(self, subgraph: Subgraph) -> None:
        self.subgraph = subgraph


class ExecutionMiss(object):
    pass


class ExecutionComplete(object):
    pass


ExecutionResult = Union[ExecutionHit, ExecutionMiss, ExecutionComplete]


class ExecutionEvent(object):
    def __init__(self, key: str, subgraph: Subgraph) -> None:
        self.key = key
        self.subgraph = subgraph


def parse_s3_event(event) -> str:
    # Retrieve body of sns message
    # Decode json body of sns message
    print('event is {}'.format(event))
    msg = json.loads(event['body'])['Message']
    msg = json.loads(msg)

    record = msg['Records'][0]

    bucket = record['s3']['bucket']['name']
    key = record['s3']['object']['key']
    return download_s3_file(bucket, key)

def download_s3_file(bucket, key) -> str:
    print('downloading {} {}'.format(bucket, key))
    s3 = boto3.resource('s3')
    obj = s3.Object(bucket, key)
    return obj.get()['Body'].read()


def execute_file(file: str, graph: Subgraph, sender):
    print('executing file')
    print(file)
    exec(file, globals())
    print('File executed: {}'.format(analyzer(graph, sender)))  # type: ignore


def emit_event(event: ExecutionHit) -> None:
    print('emitting event')
    event_s = event.SerializeToString()
    event_hash = hashlib.sha256(event_s)
    key = base64.urlsafe_b64encode(event_hash)

    s3 = boto3.resource('s3')

    obj = s3.Object('grapl-analyzer-hits', key)
    obj.put(Body=event_s)

def lambda_handler(events: Any, context: Any) -> None:
    # Parse sns message
    print('handling')
    print(events)
    print(context)
    for event in events['Records']:
        data = parse_s3_event(event)

        message = json.loads(data)

        # TODO: Use env variable for s3 bucket
        analyzer = download_s3_file('grapl-analyzers-bucket', message['key'])
        subgraph = Subgraph(bytes(message['subgraph']))

        # TODO: Validate signature of S3 file
        print('creating queue')
        rx, tx = Pipe(duplex=False)  # type: Tuple[Connection, Connection]
        print('creating process')
        p = Process(target=execute_file, args=(analyzer, subgraph, tx))
        print('running process')
        p.start()


        while True:
            print('waiting for results')
            result = rx.recv()  # type: Optional[ExecutionResult]
            # TODO: If we receive no result, assume process failed unexpectedly
            assert result

            if isinstance(result, ExecutionComplete):
                break

            # emit any hits to an S3 bucket
            if isinstance(result, ExecutionHit):
                print('emitting event for result: {}'.format(result))
                emit_event(result)

        p.join()


if __name__ == '__main__':
    print('started')