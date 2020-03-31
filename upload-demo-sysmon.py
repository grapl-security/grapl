#!/usr/bin/env python

try:
    from typing import Any, Dict, Union, Optional
except:
    pass

from datetime import datetime
import time
import string
import boto3
import random
import zstd
import sys
import json


def rand_str(l):
    # type: (int) -> str
    return ''.join(random.choice(string.ascii_uppercase + string.digits)
                   for _ in range(l))

def into_sqs_message(bucket: str, key: str) -> str:
    return json.dumps(
        {
            'Records': [
                {
                    'eventTime': datetime.utcnow().isoformat(),
                    'principalId': {
                        'principalId': None,
                    },
                    'requestParameters': {
                        'sourceIpAddress': None,
                    },
                    'responseElements': {},
                    's3': {
                        'schemaVersion': None,
                        'configurationId': None,
                        'bucket': {
                            'name': bucket,
                            'ownerIdentity': {
                                'principalId': None,
                            }
                        },
                        'object': {
                            'key': key,
                            'size': 0,
                            'urlDecodedKey': None,
                            'versionId': None,
                            'eTag': None,
                            'sequencer': None
                        }
                    }

                }
            ]
        }
    )

def main(prefix):

    sqs = None
    if prefix == 'local-grapl':
        s3 = boto3.client(
            's3',
            endpoint_url="http://localhost:9000",
            aws_access_key_id='minioadmin',
            aws_secret_access_key='minioadmin',
        )
        sqs = boto3.client('sqs', endpoint_url="http://localhost:9324")

        sqs.purge_queue(
            QueueUrl='http://localhost:9324/queue/sysmon-graph-generator-queue',

        )

    else:
        raise NotImplementedError

    with open('./eventlog.xml', 'rb') as b:
        body = b.readlines()
        body = [line for line in body]

    def chunker(seq, size):
        return [seq[pos:pos + size] for pos in range(0, len(seq), size)]

    for chunks in chunker(body, 50):
        c_body = zstd.compress(b"\n".join(chunks).replace(b"\n\n", b"\n"), 4)
        epoch = int(time.time())

        key = str(epoch - (epoch % (24 * 60 * 60))) + "/sysmon/" + str(epoch) + rand_str(3)
        s3.put_object(
            Body=c_body,
            Bucket="{}-sysmon-log-bucket".format(prefix),
            Key=key
        )

        if sqs:
            sqs.send_message(
                QueueUrl='http://localhost:9324/queue/sysmon-graph-generator-queue',
                MessageBody=into_sqs_message(
                    bucket="{}-sysmon-log-bucket".format(prefix),
                    key=key
                )
            )


    print(time.ctime())

if __name__ == '__main__':

    if len(sys.argv) != 2:
        raise Exception("Provide bucket prefix as first argument")
    else:
        main(sys.argv[1])
