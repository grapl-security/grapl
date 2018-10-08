print('importing base64')
import base64

print('importing threading')
import threading

print('importing boto3')
import boto3
print('importing json')
import json

print('importing subgraph_merge_event_pb2')
import subgraph_merge_event_pb2
print('importing pydgraph')
import pydgraph

try:
    from typing import Optional, Any, Dict, Union, Callable, TypeVar, Generic, List
except:
    pass

T = TypeVar('T')
U = TypeVar('U')


class Option(Generic[T]):
    def __init__(self, t):
        # type: (Optional[T]) -> None
        self.t = t

    def or_else(self, default):
        # type: (T) -> T
        if self.t is not None:
            return self.t
        else:
            return default

    def or_none(self):
        # type: () -> Optional[T]
        return self.t

    def map(self, fn):
        # type: (Callable[[T], U]) -> Option[U]
        if self.t is not None:
            return Option(fn(self.t))
        else:
            return Option(None)

    def __iter__(self):
        if self.t is not None:
            t = self.t
            self.t = None
            return t
        else:
            raise StopIteration


class File(object):

    def __init__(self,
                 uid,
                 path,
                 create_time,
                 delete_time,
                 node_key,
                 asset_id,
                 creator,
                 ):
        self.uid = uid  # type: str
        self.path = path  # type: str
        self.create_time = create_time  # type: int
        self.delete_time = delete_time  # type: Option[int]
        self.node_key = node_key  # type: str
        self.creator = creator  # type: Option[Process]
        self.asset_id = asset_id  # type: str

    def to_dict(self):
        # type: () -> Dict[str, Any]
        return {
            "File": {
                'uid': self.uid,
                'path': self.path,
                'create_time': self.create_time,
                'delete_time': self.delete_time.or_none(),
                'node_key': self.node_key,
                'asset_id': self.asset_id,
                'creator': self.creator
                    .map(Process.to_dict)
                    .map(lambda p: p["Process"])
                    .or_none(),
            }
        }

    def to_json(self):
        return json.dumps(
            self.to_dict()
        )

    @staticmethod
    def from_json(j):
        # type: (Union[str, Dict[str, Any]]) -> File
        if isinstance(j, str):
            s = json.loads(j)  # type: Dict[str, Any]
        else:
            s = j

        return File(
            path=s['path'],
            uid=s['uid'],
            create_time=int(s['create_time']),
            delete_time=Option(s.get('delete_time')).map(int),
            node_key=s['node_key'],
            asset_id=s['asset_id'],
            creator=Option(s.get('creator')).map(Process.from_json),
        )


class Process(object):
    def __init__(self,
                 uid,
                 pid,
                 create_time,
                 terminate_time,
                 node_key,
                 asset_id,
                 image_name,
                 bin_file,
                 children
                 ):
        self.uid = uid  # type: str
        self.pid = pid  # type: int
        self.create_time = create_time  # type: int
        self.terminate_time = terminate_time  # type: Option[int]
        self.node_key = node_key  # type: str
        self.asset_id = asset_id  # type: str
        self.image_name = image_name  # type: Option[str]
        self.bin_file = bin_file  # type: Option[File]
        self.children = children  # type: List[Process]

    @staticmethod
    def from_json(j):
        # type: (Union[str, Dict[str, Any]]) -> Process
        if isinstance(j, str):
            s = json.loads(j)  # type: Dict[str, Any]
        else:
            s = j

        return Process(
            uid=s['uid'],
            pid=int(s['pid']),
            create_time=int(s['create_time']),
            terminate_time=Option(s.get('terminate_time')).map(int),
            node_key=s['node_key'],
            asset_id=s['asset_id'],
            image_name=Option(s.get('image_name')),
            bin_file=Option(s.get('bin_file')).map(File.from_json),
            children=[Process.from_json(child) for child in s.get('children', [])]
        )

    def to_dict(self):
        # type: () -> Dict[str, Any]
        return {
            "Process": {
                'uid': self.uid,
                'pid': self.pid,
                'create_time': self.create_time,
                'terminate_time': self.terminate_time.or_none(),
                'node_key': self.node_key,
                'asset_id': self.asset_id,
                'image_name': self.image_name.or_none(),
                'bin_file': self.bin_file.map(File.to_dict).or_none(),
                'children': [child.to_dict()["Process"] for child in self.children],
            }
        }

    def to_json(self):
        return json.dumps(
            self.to_dict()
        )


class RootNode(object):
    def __init__(self, node):
        # type: (Union[File, Process]) -> None
        self.node = node

    @staticmethod
    def from_json(j):
        # type: (Union[str, Dict[str, Any]]) -> RootNode
        print('parsing json')
        if isinstance(j, str):
            d = json.loads(j)  # type: Dict[str, Any]
        else:
            d = j

        print('json parsed')

        if d.get('pid'):
            print('parsing process node')
            return RootNode(Process.from_json(j))
        elif d.get('path'):
            print('parsing file node')
            return RootNode(File.from_json(j))

        raise Exception("Unsupported node type: {}".format(j))

    def to_dict(self):
        # type: () -> Dict[str, Any]
        return self.node.to_dict()

    def to_json(self):
        # type: () -> Dict[str, Any]
        return self.node.to_json()


def _get_topic_arn(sns, name):
    # type: (Any, str) -> str
    return sns.create_topic(Name=name)['TopicArn']


def publish(sns, arn, message):
    # type: (Any, str, Any) -> None
    sns.publish(
        TargetArn=arn,
        Message=json.dumps({'default': message}),
        MessageStructure='json'
    )


def emit_incident(context, matches, sns_client=None):
    # type: (Any, List[RootNode], Option[Any]) -> None

    sns = boto3.client('sns')

    # Chunk up messages since SNS has a size limit
    n = 50
    matches = [match.to_dict() for match in matches]
    matches_chunks = (matches[i:i+n] for i in range(0, len(matches), n))

    arn = "arn:aws:sns:us-east-1:251074890252:grapl-stack-graplincidenttopic79D8E386-OVD7AYQV6Y03"

    for matches in matches_chunks:
        print('publishing matches')
        publish(sns, arn, json.dumps(matches))


def get_queue_url(sqs, queue_name):
    # type: (Any, str) -> str
    return sqs.get_queue_url(
        QueueName=queue_name
    )['QueueUrl']


def ack_msg(message, sqs_client=None):
    # type: (Dict[str, Any], Optional[Any]) -> None
    sqs = sqs_client or boto3.client('sqs')  # type: Any
    print(message)
    receipt_id = message["receiptHandle"]
    queue_name = message["eventSourceARN"].split(":")[-1]

    queue_url = get_queue_url(sqs, queue_name)

    sqs.delete_message(
        QueueUrl=queue_url,
        ReceiptHandle=receipt_id
    )


def word_macro_matcher(earliest, latest):
    # type: (int, int) -> List[RootNode]
    if earliest != 0:
        earliest = earliest - 1
    query_str = """    
    {{
      q(func: eq(image_name, "word")) 
      @filter(gt(create_time, {}) AND lt(create_time, {}))
      {{
        uid, pid, create_time, image_name, terminate_time, node_key, asset_id
        children {{
            uid, pid, create_time, image_name, terminate_time, node_key, asset_id
        }},
        bin_file {{
          uid,
          create_time,
          delete_time,
          asset_id,
          node_key
        }}
      }}
    }}
    """.format(earliest, latest + 1)

    print(query_str)
    client = pydgraph.DgraphClient(pydgraph.DgraphClientStub('db.mastergraph:9080'))

    res = json.loads(client.txn().query(query_str).json)
    print(res)
    res = res['q']
    res = res if isinstance(res, List) else [res]
    return [RootNode.from_json(s) for s in res]


def lambda_handler(event, context):
    print('starting lambda_handler')
    join_handles = []
    for message in event['Records']:
        print(message)
        def ex():
            try:
                body=json.loads(message["body"])["Message"]
                print(base64.b64decode(body))
                print(type(body))
                subgraph_merge_event = subgraph_merge_event_pb2\
                    .SubgraphMerged()
                subgraph_merge_event.ParseFromString(base64.b64decode(body))
                print(subgraph_merge_event)
                print(subgraph_merge_event.earliest)
                print(subgraph_merge_event.latest)
                if (subgraph_merge_event.earliest is None or
                        subgraph_merge_event.latest is None):
                    print("not earliest and latest")
                    return
                earliest = subgraph_merge_event.earliest
                latest = subgraph_merge_event.latest
                print("{} {}".format(earliest, latest))
                matches = word_macro_matcher(earliest, latest)
                print(matches)
                if matches:
                    print('emit_incident')
                    emit_incident(context, matches)
                    print('acking message')
                    ack_msg(message)

                return 1
            except Exception as e:
                print(e)
                return e
        join_handles.append(threading.Thread(target=ex))

    [t.start() for t in join_handles]
    results = [t.join() for t in join_handles]
    results = [t for t in results if t]
    if results:
        raise results[0]

