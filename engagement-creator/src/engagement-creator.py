import json
import os

from typing import *

import boto3
from collections import defaultdict
from grapl_analyzerlib.prelude import NodeView, FileView, ProcessView
from grapl_analyzerlib.nodes.lens_node import CopyingDgraphClient, LensView
from pydgraph import DgraphClient, DgraphClientStub


def parse_s3_event(event: Dict[str, Any]) -> str:
    # Retrieve body of sns message
    # Decode json body of sns message
    print('event is {}'.format(event))
    msg = json.loads(event['body'])['Message']
    msg = json.loads(msg)

    record = msg['Records'][0]

    bucket = record['s3']['bucket']['name']
    key = record['s3']['object']['key']
    return download_s3_file(bucket, key)


def download_s3_file(bucket: str, key: str) -> str:
    key = key.replace("%3D", "=")
    print('Downloading s3 file from: {} {}'.format(bucket, key))
    s3 = boto3.resource('s3')
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


def attach_risk(client: DgraphClient, node: Union[FileView, ProcessView], analyzer_name: str, risk_score: int):
    txn = client.txn(read_only=False)
    try:
        query = """
            query q0($a: string, $b: string)
            {
            
              n as var(func: eq(node_key, $a), first: 1) {
                uid
              }
            
              q0(func: uid(n), first: 1)
              {
                uid,
                risks @filter(
                    eq(analyzer_name, $b)
                )
                {
                    uid
                }
              }
            }
            """

        variables = {
            '$a': node.node_key,
            '$b': analyzer_name
        }
        txn = client.txn(read_only=False)
        res = json.loads(txn.query(query, variables=variables).json)

        if res['q0'] and res['q0'][0].get('risks'):
            return

        mutation = {
            "uid": res['q0'][0]['uid'],
            "risks": {
                'analyzer_name': analyzer_name,
                'risk_score': risk_score
            }
        }

        print(f"mutation: {mutation}")

        txn.mutate(set_obj=mutation, commit_now=True)
    finally:
        txn.discard()


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


def upsert(client: DgraphClient, node_dict: Dict[str, Any]) -> None:
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
            # node_dict = {**node_dict, **res[0]}

        mutation = node_dict
        print(f"Mutating with : {mutation}")

        mut_res = txn.mutate(set_obj=mutation, commit_now=True)
        print(mut_res)

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
                    expand(_forward_),
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


def lambda_handler(events: Any, context: Any) -> None:
    mg_alpha_names = os.environ['MG_ALPHAS'].split(",")
    eg_alpha_names = os.environ['EG_ALPHAS'].split(",")

    mg_client_stubs = [DgraphClientStub('{}:9080'.format(name)) for name in mg_alpha_names]
    eg_client_stubs = [DgraphClientStub('{}:9080'.format(name)) for name in eg_alpha_names]

    eg_client = DgraphClient(*eg_client_stubs)
    mg_client = DgraphClient(*mg_client_stubs)

    cclient = CopyingDgraphClient(src_client=mg_client, dst_client=eg_client)

    for event in events['Records']:
        data = parse_s3_event(event)
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
            attach_risk(eg_client, node.node, analyzer_name, risk_score)

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

