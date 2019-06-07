import os
import boto3
import json

from typing import Any, List, Dict, Union, TypeVar, Optional

from collections import defaultdict
from grapl_analyzerlib.entities import ProcessView, FileView
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


class EngagementCopier(object):

    def __init__(self, mg_client: DgraphClient, eg_client: DgraphClient):
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
                # TODO: Merge lists of properties together
                node = {**props, **node[0]}

            res = txn.mutate(set_obj=node, commit_now=True)
            uids = res.uids
            uid = uids['blank-0']
        finally:
            txn.discard()

        return uid

    def copy_node(self, node_uid: str) -> str:
        print('Copying node: {}'.format(node_uid))
        query = """
            query q0($a: string)
            {
            
              q as var(func: uid($a)) {
                  pred as _predicate_
              }
            
              q0(func: uid(q))
              {
                  expand(val(pred))
              }
            }
            """

        res = json.loads(self.mg_client.txn(read_only=True)
                         .query(query, variables={'$a': node_uid}).json)
        print(f'res {res}')
        # We assume the node exists in the master graph
        node = res['q0'][0]

        # Prep node for upsert into engagement
        node.pop('uid', None)

        # Insert node into engagement-graph
        return EngagementCopier.upsert(self.eg_client, node['node_key'], node)

    def copy_edge(self, from_uid: str, edge_name: str, to_uid: str):
        mut = {
            'uid': from_uid,
            edge_name: {'uid': to_uid}
        }

        print('mutating')
        res = self.eg_client.txn(read_only=False).mutate(set_obj=mut, commit_now=True)
        print('edge mutation result is: {}'.format(res))


def get_root(nodes: List[Dict[str, Any]]) -> Dict[str, Any]:
    root = None
    for node in nodes:
        if node.get('root'):
            root = node
            break
    assert root, 'There must be a root node'
    return root


def should_throttle(
        analyzer_name: str,
        root_node_key: str,
        dgraph_client: DgraphClient
) -> bool:
    query = """
            query q0($a: string, $n: string)
            {
              q0(func: eq(node_key, $n), first: 1) @cascade
              {
                uid,
                ~scope {
                    uid,
                    risks @filter(eq(analyzer_name, $a)) {
                        uid
                    }
                }
              }
            }
            """

    variables = {
        '$a': analyzer_name,
        '$n': root_node_key
    }

    res = json.loads(dgraph_client.txn(read_only=True).
                     query(query, variables=variables).json)
    if res['q0']:
        return True
    return False


def into_process_views(dgraph_client: DgraphClient, raw_scope) -> List[Union[ProcessView, FileView]]:
    scope = []
    for scoped_node in raw_scope:
        if scoped_node.get('process_id'):
            node = ProcessView(dgraph_client=dgraph_client, node_key=scoped_node['node_key'])
        elif scoped_node.get('file_path'):
            node = FileView(dgraph_client=dgraph_client, node_key=scoped_node['node_key'])
        else:
            raise Exception('fInvalid scoped node type: {scoped_node}')
        scope.append(node)

    return scope


EV = TypeVar('EV', bound='EngagementView')


class EngagementView(object):
    def __init__(
            self,
            client: DgraphClient,
            lens: str,
            uid: str,
            scope: Optional[List[Union[ProcessView, FileView]]] = None
    ):
        self.client = client
        self.lens = lens
        self.uid = uid
        self.scope = scope or []

    @staticmethod
    def get(client, lens: str) -> Optional[EV]:
        query = """
        query q0($a: string)
        {
          p as var(func: eq(lens, $a)) {
          }
        
          q0(func: uid(p))
          {
            uid,
            score,
            lens,
            scope {
                node_key,
                uid,
                process_id,
                file_path
            }
          }
        }
        """

        txn = client.txn(read_only=True)

        try:
            res = json.loads(txn.query(query, variables={'$a': lens}).json)
            node = res['q0']
            print(f'node is {node}')
            if node:
                return EngagementView(client,
                                      lens=node[0]['lens'],
                                      uid=node[0]['uid'],
                                      scope=into_process_views(client, node[0].get('scope', [])),
                                      )
            else:
                return None
        finally:
            txn.discard()


    @staticmethod
    def get_or_create(client, lens: str) -> EV:
        query = """
        query q0($a: string)
        {
          p as var(func: eq(lens, $a)) {
          }
        
          q0(func: uid(p))
          {
            uid,
            score,
            lens,
            scope {
                node_key,
                uid,
                process_id,
                file_path
            }
          }
        }
        """

        txn = client.txn(read_only=False)

        try:
            res = json.loads(txn.query(query, variables={'$a': lens}).json)
            node = res['q0']
            print(f'node is {node}')
            if node:
                return EngagementView(client,
                                      lens=node[0]['lens'],
                                      uid=node[0]['uid'],
                                      scope=into_process_views(client, node[0].get('scope', [])),
                                      )
            else:
                node = {'lens': lens, 'score': 0}
            res = txn.mutate(set_obj=node, commit_now=True)
            uids = res.uids
            print(f'lens upsert uids {uids}')
            uid = uids['blank-0']
            return EngagementView(client,
                                  uid=uid,
                                  lens=lens,
                                  scope=[],
                                  )
        finally:
            txn.discard()

    def attach_scope(self, root_node: Union[ProcessView, FileView]) -> EV:
        self.scope.append(root_node)
        txn = self.client.txn(read_only=False)

        try:
            mutation = {
                "uid": self.uid,
                "scope": {
                    "uid": root_node.get_uid()
                }
            }

            print(f"mutation: {mutation}")

            txn.mutate(set_obj=mutation, commit_now=True)
        finally:
            txn.discard()

        return self

    def recalculate_score(self) -> int:
        query = """
            query q0($a: string, $b: string)
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
            '$a': self.lens,
        }
        txn = self.client.txn(read_only=False)
        res = json.loads(txn.query(query, variables=variables).json)

        risk_map = defaultdict(list)
        for root_node in res['q0'][0]['scope']:
            for risk in root_node['risks']:
                risk_map['node_key'].append(risk)

        risk_score = 0

        for node_key, risks in risk_map.items():
            node_risk = 0
            for risk in risks:
                node_risk += risk['risk_score']
            risk_multiplier = (0.10 * (len(risks) - 1))
            node_risk = node_risk + (node_risk * risk_multiplier)
            risk_score += node_risk

        self.set_score(risk_score, txn=txn)

        return risk_score

    def set_score(self, new_score: int, txn=None) -> EV:
        if not txn:
            txn = self.client.txn(read_only=False)

        try:
            mutation = {
                "uid": self.uid,
                "score": new_score
            }

            print(f"mutation: {mutation}")

            txn.mutate(set_obj=mutation, commit_now=True)
        finally:
            txn.discard()

        return self

def attach_risk(client: DgraphClient, node: Dict[str, Any], analyzer_name: str, risk_score: int):
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
            '$a': node['node_key'],
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

def lambda_handler(events: Any, context: Any) -> None:
    mg_alpha_names = os.environ['MG_ALPHAS'].split(",")
    eg_alpha_names = os.environ['EG_ALPHAS'].split(",")

    mg_client_stubs = [DgraphClientStub('{}:9080'.format(name)) for name in mg_alpha_names]
    eg_client_stubs = [DgraphClientStub('{}:9080'.format(name)) for name in eg_alpha_names]

    eg_client = DgraphClient(*eg_client_stubs)
    mg_client = DgraphClient(*mg_client_stubs)

    uid_map = {}
    for event in events['Records']:
        print('Copying engagement')
        data = parse_s3_event(event)
        incident_graph = json.loads(data)

        analyzer_name = incident_graph['analyzer_name']
        nodes = incident_graph['nodes']
        edges = incident_graph['edges']
        risk_score = incident_graph['risk_score']

        print(f'nodes {nodes}')
        print(f'edges {edges}')
        # Key is root node + analyzer_name
        root = get_root(nodes.values())

        if should_throttle(analyzer_name, root['node_key'], eg_client):
            print('Throttling: {}'.format(nodes))
            continue

        # Upsert all of the nodes
        # If nodes have a list field, merge it
        # In particular, merge the 'analyzer_names' list

        copier = EngagementCopier(mg_client, eg_client)

        print('node_refs: {}'.format(nodes))
        print('edges: {}'.format(edges))
        for node in nodes.values():
            node.pop('root', None)
            node.pop('type', None)

            new_uid = copier.copy_node(node['uid'])
            uid_map[node['node_key']] = new_uid
            print('new_uid: {}'.format(new_uid))

        for edge_list in edges.values():
            for edge in edge_list:
                copier.copy_edge(uid_map[edge['from']], edge['edge_name'], uid_map[edge['to']])
        print('Copied engagement successfully')

        if root['node_type'] == 'Process':
            root_view = ProcessView(dgraph_client=eg_client, node_key=root['node_key'])
        elif root['node_type'] == 'File':
            root_view = FileView(dgraph_client=eg_client, node_key=root['node_key'])
        else:
            raise Exception(f"Invalid root node. Missing 'type': {root}.")

        asset_id = root_view.get_asset_id()

        engagement = EngagementView.get_or_create(eg_client, lens=asset_id)

        engagement.attach_scope(root_view)

        for node in nodes.values():
            attach_risk(
                eg_client, node, analyzer_name, risk_score
            )

        score = engagement.recalculate_score()
        print(f'Engagement has score: {score}')

    print('Engagement creation was successful')
