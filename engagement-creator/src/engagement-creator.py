import boto3
import json
from uuid import uuid4, UUID

from typing import Any, Dict, List, Tuple

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
    node_attr_map = {
        'Process': "image_name, pid"  # todo: fill this out
    }

    def __init__(self, engagement_id: UUID, mg_client: DgraphClient, eg_client: DgraphClient):
        self.engagement_id = str(engagement_id)
        self.mg_client = mg_client
        self.eg_client = eg_client

    def copy_node(self, node_uid: str, node_type: str) -> str:
        """

        :param node_uid:
        :param node_type:
        :return:
        """
        print('Copying {} node: {}'.format(node_type, node_uid))
        query = """
            query q0($a: string)
            {
              q0(func: uid($a))
              {
                uid,
                node_key,
              }
            }
            """

        res = json.loads(self.mg_client.query(query, variables={'$a': node_uid}).json)
        print('res {}'.format(res))
        node = res['q0'][0]

        # Insert node into engagement-graph

        mut = {
            'node_key': node['node_key'],
            'engagement_key': str(self.engagement_id),
            # **{attr: node[attr] for attr in NodeCopier.node_attr_map.get(node_type)}
        }

        print('mutating')
        res = self.eg_client.txn().mutate(set_obj=mut, commit_now=True)
        print(res)
        return res.uids[0]


    def copy_edge(self, from_uid: str, edge_name: str, to_uid: str):
        mut = {
            'uid': from_uid,
            edge_name: {'uid': to_uid}
        }

        print('mutating')
        self.eg_client.txn().mutate(set_obj=mut, commit_now=True)


def create_process_schema(eg_client: DgraphClient):
    schema = \
        'node_key: string @index(hash) .\n' +\
        'engagement_key: string @index(hash) .\n' +\
        'pid: int @index(int) .\n'

    op = pydgraph.Operation(schema=schema)
    eg_client.alter(op)


def lambda_handler(events: Any, context: Any) -> None:
    mg_client = DgraphClient(DgraphClientStub('db.mastergraph:9080'))
    eg_client = DgraphClient(DgraphClientStub('db.engagementgraph:9080'))
    engagement_id = uuid4()

    create_process_schema(eg_client)

    copier = NodeCopier(engagement_id, mg_client, eg_client)

    uid_map = {}
    for event in events['Records']:
        print('Copying engagement')
        data = parse_s3_event(event)
        incident_graph = json.loads(data)
        node_refs = incident_graph['node_refs']
        edges = incident_graph['edges']

        for node_ref in node_refs:
            new_uid = copier.copy_node(node_ref['uid'], node_ref['node_type'])
            uid_map[node_ref['uid']] = new_uid

        for edge in edges:
            copier.copy_edge(uid_map[edge[0]], edge[1], uid_map[edge[2]])
        print('Copied engagement successfully')

    print('Engagement creation was successful')
