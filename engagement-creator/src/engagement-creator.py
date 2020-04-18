import json
import os
import time
import traceback

from collections import defaultdict
from typing import *

import boto3
from grapl_analyzerlib.nodes.lens_node import CopyingDgraphClient, LensView
from grapl_analyzerlib.prelude import NodeView, FileView, ProcessView
from pydgraph import DgraphClient, DgraphClientStub

IS_LOCAL = bool(os.environ.get('IS_LOCAL', False))


def parse_s3_event(s3, event) -> str:
    # Retrieve body of sns message
    # Decode json body of sns message
    print("event is {}".format(event))
    # msg = json.loads(event["body"])["Message"]
    # msg = json.loads(msg)

    bucket = event["s3"]["bucket"]["name"]
    key = event["s3"]["object"]["key"]
    return download_s3_file(s3, bucket, key)


def download_s3_file(s3, bucket: str, key: str) -> str:
    key = key.replace("%3D", "=")
    print('Downloading s3 file from: {} {}'.format(bucket, key))
    obj = s3.Object(bucket, key)
    return obj.get()['Body'].read()


def create_edge(client: DgraphClient, from_uid: str, edge_name: str, to_uid: str) -> None:
    if edge_name[0] == '~':
        mut = {
            'uid': to_uid,
            edge_name[1:]: {'uid': from_uid}
        }

    else:
        mut = {
            'uid': from_uid,
            edge_name: {'uid': to_uid}
        }

    txn = client.txn(read_only=False)
    try:
        res = txn.mutate(set_obj=mut, commit_now=True)
        print('edge mutation result is: {}'.format(res))
    finally:
        txn.discard()


def attach_risk(client: DgraphClient, node_key: str, node_uid: str, analyzer_name: str, risk_score: int) -> None:

    risk_node = {
        'node_key': node_key + analyzer_name,
        'analyzer_name': analyzer_name,
        'risk_score': risk_score,
        'dgraph.type': 'Risk',
    }

    risk_node_uid = upsert(client, risk_node)

    create_edge(client, node_uid, 'risks', risk_node_uid)



def recalculate_score(client: DgraphClient, lens: LensView) -> int:
    query = """
        query q0($a: string)
        {
          q0(func: eq(lens, $a), first: 1) @cascade
          {
            uid,
            scope {
                node_key,
                risks {
                    analyzer_name,
                    risk_score
                }
            }
          }
        }
        """

    variables = {
        '$a': lens.lens,
    }
    txn = client.txn(read_only=False)
    risk_score = 0
    try:
        res = json.loads(txn.query(query, variables=variables).json)['q0']

        redundant_risks = set()
        risk_map = defaultdict(list)
        if not res:
            logging.warning("Received an empty response for risk query")
            return 0
        for root_node in res[0]['scope']:
            for risk in root_node['risks']:
                if risk['analyzer_name'] in redundant_risks:
                    continue
                redundant_risks.add(risk['analyzer_name'])
                risk_map[risk['analyzer_name']].append(risk)

        for risks in risk_map.values():
            node_risk = 0
            for risk in risks:
                node_risk += risk['risk_score']
            risk_multiplier = (0.10 * ((len(risks) or 1) - 1))
            node_risk = node_risk + (node_risk * risk_multiplier)
            risk_score += node_risk

        set_score(client, lens.uid, risk_score, txn=txn)
    finally:
        txn.discard()

    return risk_score


def set_score(client: DgraphClient, uid: str, new_score: int, txn=None) -> None:
    if not txn:
        txn = client.txn(read_only=False)

    try:
        mutation = {
            "uid": uid,
            "score": new_score
        }

        txn.mutate(set_obj=mutation, commit_now=True)
    finally:
        txn.discard()


def set_property(client: DgraphClient, uid: str, prop_name: str, prop_value):
    print(f'Setting property {prop_name} as {prop_value} for {uid}')
    txn = client.txn(read_only=False)

    try:
        mutation = {
            "uid": uid,
            prop_name: prop_value
        }

        txn.mutate(set_obj=mutation, commit_now=True)
    finally:
        txn.discard()


def upsert(client: DgraphClient, node_dict: Dict[str, Any]) -> str:
    if node_dict.get('uid'):
        node_dict.pop('uid')
    node_dict['uid'] = '_:blank-0'
    node_key = node_dict['node_key']
    print(f"INFO: Upserting node: {node_dict}")
    query = f"""
        {{
            q0(func: eq(node_key, "{node_key}")) {{
                    uid,
                    dgraph.type,
            }}
        }}
        """
    txn = client.txn(read_only=False)

    try:
        res = json.loads(txn.query(query).json)['q0']

        if res:
            node_dict['uid'] = res[0]['uid']
            node_dict = {**node_dict, **res[0]}

        mutation = node_dict

        mut_res = txn.mutate(set_obj=mutation, commit_now=True)
        new_uid = node_dict.get('uid') or mut_res.uids["blank-0"]
        return new_uid

    finally:
        txn.discard()


def copy_node(
        mgclient: DgraphClient,
        egclient: DgraphClient,
        node_key: str,
        init_node: Optional[Dict[str, Any]] = None
) -> None:
    if not init_node:
        init_node = dict()
    assert init_node is not None

    query = f"""
        {{
            q0(func: eq(node_key, "{node_key}")) {{
                    uid,
                    expand(_all_),
                    dgraph.type    
            }}
        }}
        """
    txn = mgclient.txn(read_only=True)

    try:
        res = json.loads(txn.query(query).json)['q0']
    finally:
        txn.discard()

    if not res:
        raise Exception("ERROR: Can not find res")

    print(f"Copy query result: {res}")

    raw_to_copy = {**res[0], **init_node}

    return upsert(egclient, raw_to_copy)


def get_s3_client():
    if IS_LOCAL:
        return boto3.resource(
            "s3",
            endpoint_url="http://s3:9000",
            aws_access_key_id='minioadmin',
            aws_secret_access_key='minioadmin',
        )
    else:
        return boto3.resource("s3")


def lambda_handler(events: Any, context: Any) -> None:
    mg_alpha_names = os.environ['MG_ALPHAS'].split(",")
    mg_alpha_port = os.environ.get('MG_ALPHA_PORT', '9080')
    eg_alpha_names = os.environ['EG_ALPHAS'].split(",")
    eg_alpha_port = os.environ.get('EG_ALPHA_PORT', '9080')

    mg_client_stubs = [DgraphClientStub(f'{name}:{mg_alpha_port}') for name in mg_alpha_names]
    eg_client_stubs = [DgraphClientStub(f'{name}:{eg_alpha_port}') for name in eg_alpha_names]

    eg_client = DgraphClient(*eg_client_stubs)
    mg_client = DgraphClient(*mg_client_stubs)

    cclient = CopyingDgraphClient(src_client=mg_client, dst_client=eg_client)

    s3 = get_s3_client()
    for event in events['Records']:
        if not IS_LOCAL:
            event = json.loads(event['body'])['Records'][0]

        data = parse_s3_event(s3, event)
        incident_graph = json.loads(data)

        analyzer_name = incident_graph['analyzer_name']
        nodes = incident_graph['nodes']
        edges = incident_graph['edges']
        risk_score = incident_graph['risk_score']

        print(f'AnalyzerName {analyzer_name}, nodes: {nodes} edges: {type(edges)} {edges}')

        nodes = [NodeView.from_node_key(mg_client, n['node_key']) for n in nodes.values()]

        copied_nodes = {}
        lenses = {}  # type: Dict[str, LensView]
        for node in nodes:
            print(f'Copying node: {node}')
            # Only support asset lens for now
            copy_node(mg_client, eg_client, node.node.node_key, init_node=node.node.get_properties())
            copied_node = NodeView.from_node_key(eg_client, node.node.node_key)
            if node.as_process():
                asset_id = node.as_process().get_asset_id()
            elif node.as_file():
                asset_id = node.as_file().get_asset_id()
            else:
                if hasattr(node.node, 'asset_id'):
                    asset_id = node.node.asset_id
                else:
                    print(f'Unsupported node: {node} {node.node.node_type}')
                    continue

            print(f'Getting lens for: {asset_id}')
            lens = lenses.get(asset_id) or LensView.get_or_create(cclient, asset_id)
            lenses[asset_id] = lens

            # Attach to scope
            create_edge(eg_client, lens.uid, 'scope', copied_node.uid)

            copied_nodes[copied_node.node_key] = copied_node.uid

            # If a node shows up in a lens all of its connected nodes should also show up in that lens
            for edge_list in edges.values():
                for edge in edge_list:
                    from_uid = copied_nodes.get(edge['from'])
                    to_uid = copied_nodes.get(edge['to'])
                    if not from_uid:
                        copy_node(mg_client, eg_client, edge['from'])
                        copied_node = NodeView.from_node_key(eg_client, edge['from'])
                    if not to_uid:
                        copy_node(mg_client, eg_client, edge['to'])
                        copied_node = NodeView.from_node_key(eg_client, edge['to'])

                    create_edge(eg_client, lens.uid, 'scope', copied_node.uid)

        for node in nodes:
            node_uid = copied_nodes[node.node.node_key]
            attach_risk(eg_client, node.node.node_key, node_uid, analyzer_name, risk_score)

        for edge_list in edges.values():
            for edge in edge_list:
                from_uid = copied_nodes.get(edge['from'])
                edge_name = edge['edge_name']
                to_uid = copied_nodes.get(edge['to'])
                if not from_uid:
                    copy_node(mg_client, eg_client, edge['from'])
                    copied_node = NodeView.from_node_key(eg_client, edge['from'])
                    from_uid = copied_node.uid
                if not to_uid:
                    copy_node(mg_client, eg_client, edge['to'])
                    copied_node = NodeView.from_node_key(eg_client, edge['to'])
                    to_uid = copied_node.uid

                create_edge(eg_client, from_uid, edge_name, to_uid)

        for lens in lenses.values():
            recalculate_score(eg_client, lens)



if IS_LOCAL:
    os.environ['MG_ALPHAS'] = 'master_graph'
    os.environ['EG_ALPHAS'] = 'engagement_graph'
    os.environ['EG_ALPHA_PORT'] = '9080'

    sqs = boto3.client(
        'sqs',
        region_name="us-east-1",
        endpoint_url="http://sqs.us-east-1.amazonaws.com:9324",
        aws_access_key_id='dummy_cred_aws_access_key_id',
        aws_secret_access_key='dummy_cred_aws_secret_access_key',
    )

    while True:
        try:
            res = sqs.receive_message(
                QueueUrl="http://sqs.us-east-1.amazonaws.com:9324/queue/engagement-creator-queue",
                WaitTimeSeconds=10,
                MaxNumberOfMessages=10,
            )

            messages = res.get('Messages', [])
            if not messages:
                print('queue was empty')

            s3_events = [(json.loads(msg['Body']), msg['ReceiptHandle']) for msg in messages]
            for s3_event, receipt_handle in s3_events:
                lambda_handler(s3_event, {})

                sqs.delete_message(
                    QueueUrl="http://sqs.us-east-1.amazonaws.com:9324/queue/engagement-creator-queue",
                    ReceiptHandle=receipt_handle,
                )

        except Exception as e:
            print('mainloop exception', e)
            traceback.print_exc()
            time.sleep(2)