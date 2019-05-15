import hashlib
import time

import boto3
import json

from typing import Any, List, Dict

import os
import pydgraph
from pydgraph import DgraphClient, DgraphClientStub


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
    key = key.replace("%3D", "=")
    print('Downloading s3 file from: {} {}'.format(bucket, key))
    s3 = boto3.resource('s3')
    obj = s3.Object(bucket, key)
    return obj.get()['Body'].read()


class NodeCopier(object):

    def __init__(self, engagement_key: str, mg_client: DgraphClient, eg_client: DgraphClient):
        self.engagement_key = engagement_key
        self.mg_client = mg_client
        self.eg_client = eg_client

    @staticmethod
    def upsert(client: DgraphClient, node_key: str, props: Dict[str, str]):
        query = """
            query q0($a: string)
            {
              q0(func: eq(node_key, $a))
              {
                uid,
                expand(_all_)
              }
            }
            """

        txn = client.txn(read_only=False)

        try:
            res = json.loads(txn.query(query, variables={'$a': node_key}).json)
            node = res['q0']

            if not node:
                node = props
            else:
                node = {**props, **node[0]}

            res = txn.mutate(set_obj=node, commit_now=True)
            uids = res.uids
            print(uids)
            uid = uids['blank-0']
        finally:
            txn.discard()

        return uid

    def copy_node(self, node_uid: str) -> str:
        print('Copying node: {}'.format(node_uid))
        query = """
            query q0($a: string)
            {
              q0(func: uid($a))
              {
                expand(_all_)
              }
            }
            """

        res = json.loads(self.mg_client.query(query, variables={'$a': node_uid}).json)

        # We assume the node exists in the master graph
        node = res['q0'][0]

        # Prep node for upsert into engagement
        node.pop('uid', None)
        node['engagement_key'] = str(self.engagement_key)

        # Insert node into engagement-graph
        return NodeCopier.upsert(self.eg_client, node['node_key'], node)

    def copy_edge(self, from_uid: str, edge_name: str, to_uid: str):
        mut = {
            'uid': from_uid,
            edge_name: {'uid': to_uid}
        }

        print('mutating')
        res = self.eg_client.txn(read_only=False).mutate(set_obj=mut, commit_now=True)
        print('edge mutation result is: {}'.format(res))


def create_process_schema(eg_client: DgraphClient):
    schema = \
        'node_key: string @index(hash) .\n' +\
        'engagement_key: string @index(hash) .\n' +\
        'children: uid @reverse .\n' +\
        'pid: int @index(int) .\n'

    op = pydgraph.Operation(schema=schema)
    eg_client.alter(op)


def get_engagement_key(label: str, uids: List[str]) -> str:
    bucket = int(time.time()) - (int(time.time()) % 7200)
    hasher = hashlib.sha1(label)
    hasher.update(str(bucket).encode())
    [hasher.update(str(uid).encode()) for uid in sorted(uids)]

    return str(hasher.hexdigest())


# We need to whitelist by taking the uids, sorting, and hashing with the alert name
# For now, we 'throttle' by hour, but this should be customizable later
def should_throttle(engagement_key: str, dgraph_client: DgraphClient) -> bool:
    query = """
            query q0($a: string)
            {
              q0(func: eq(engagement_key, $a), first: 1)
              {
                uid,
              }
            }
            """

    res = json.loads(dgraph_client.query(query, variables={'$a': engagement_key}).json)
    if res['q0']:
        return True
    return False


def lambda_handler(events: Any, context: Any) -> None:
    mg_alpha_names = os.environ['MG_ALPHAS'].split(",")
    eg_alpha_names = os.environ['EG_ALPHAS'].split(",")

    mg_client_stubs = [DgraphClientStub('{}:9080'.format(name)) for name in mg_alpha_names]
    eg_client_stubs = [DgraphClientStub('{}:9080'.format(name)) for name in eg_alpha_names]

    eg_client = DgraphClient(*eg_client_stubs)
    mg_client = DgraphClient(*mg_client_stubs)

    create_process_schema(eg_client)

    uid_map = {}
    for event in events['Records']:
        print('Copying engagement')
        data = parse_s3_event(event)
        incident_graph = json.loads(data)

        label = incident_graph['label'].encode('utf-8')
        node_refs = incident_graph['node_refs']
        edges = incident_graph['edges']

        engagement_key = get_engagement_key(label, [n['uid'] for n in node_refs])

        if should_throttle(engagement_key, eg_client):
            print('Throttling: {}'.format(engagement_key))
            continue

        print('Creating engagement: {}'.format(engagement_key))
        copier = NodeCopier(engagement_key, mg_client, eg_client)

        print('node_refs: {}'.format(node_refs))
        print('edges: {}'.format(edges))
        for node_ref in node_refs:
            new_uid = copier.copy_node(node_ref['uid'])
            uid_map[node_ref['uid']] = new_uid
            print('new_uid: {}'.format(new_uid))
        for edge in edges:
            copier.copy_edge(uid_map[edge[0]], edge[1], uid_map[edge[2]])
        print('Copied engagement successfully')

    print('Engagement creation was successful')


