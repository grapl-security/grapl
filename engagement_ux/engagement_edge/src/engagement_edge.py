import sys, traceback
import json
import os
import time
from hashlib import sha256, blake2b
from hmac import compare_digest
from random import uniform
from typing import List, Dict, Any, Optional

import boto3
import jwt
import pydgraph

from grapl_analyzerlib.nodes.any_node import NodeView, raw_node_from_node_key
from grapl_analyzerlib.nodes.dynamic_node import DynamicNodeView, _DynamicNodeView
from pydgraph import DgraphClient

JWT_SECRET = os.environ['JWT_SECRET']
ORIGIN = "https://" + os.environ['BUCKET_PREFIX'] + "engagement-ux-bucket.s3.amazonaws.com"
# ORIGIN = "http://localhost:63342"
DYNAMO = None


def list_all_lenses(prefix: str) -> List[Dict[str, Any]]:
    client_stub = pydgraph.DgraphClientStub('alpha0.engagementgraphcluster.grapl:9080')
    dg_client = pydgraph.DgraphClient(client_stub)

    # DGraph query for all nodes with a 'lens' that matches the 'prefix'
    if prefix:
        query = """
            query q0($a: string)
            {
                q0(func: alloftext(lens, $a), orderdesc: score)
                {
                    uid,
                    node_key,
                    lens,
                    score
                }
            }"""

        variables = {'$a': prefix}
    else:
        query = """
            {
                q0(func: has(lens), orderdesc: score)
                {
                    uid,
                    node_key,
                    lens,
                    score
                }
            }"""

        variables = {}

    txn = dg_client.txn(read_only=True)

    try:
        res = json.loads(txn.query(query, variables=variables).json)
        return res['q0']
    finally:
        txn.discard()



def edge_in_lens(dg_client: DgraphClient, node_uid: str, edge_name: str, lens_name: str) -> List[Dict[str, Any]]:
    query = f"""
        query q0($node_uid: string, $lens_name: string)
        {{
            q0(func: uid($node_uid)) @cascade {{
                {edge_name} {{
                    uid,
                    node_key,
                    
                    ~scope @filter(eq(lens, $lens_name)) {{
                        uid
                    }}
                }}
            }}
        }}
    """

    txn = dg_client.txn(read_only=True)

    try:
        variables = {
            '$node_uid': node_uid,
            '$lens_name': lens_name
        }
        res = json.loads(txn.query(query, variables=variables).json)
        return res['q0']
    finally:
        txn.discard()


def get_lens_scope(dg_client: DgraphClient, lens: str) -> Dict[str, Any]:
    query = """
        query q0($a: string)
        {  
            q0(func: eq(lens, $a)) {
                uid,
                node_key,
                lens,
                score,
                scope {
                    uid,
                    expand(_forward_),
                    node_type: dgraph.type
                }
            }  
      }"""

    txn = dg_client.txn(read_only=True)

    try:
        variables = {'$a': lens}
        res = json.loads(txn.query(query, variables=variables).json)
        if not res['q0']:
            return {}
        return res['q0'][0]
    finally:
        txn.discard()

def get_lens_risks(dg_client: DgraphClient, lens: str) ->List[Dict[str, Any]]:
    query = """
        query q0($a: string)
        {  
            q0(func: eq(lens, $a)) @cascade {
                uid,
                node_key,
                lens,
                score,
                scope {
                    uid,
                    node_key,
                    node_type: dgraph.type
                    risks {
                        uid,
                        analyzer_name,
                        risk_score
                    }
                }
            }  
      }"""

    txn = dg_client.txn(read_only=True)

    try:
        variables = {'$a': lens}
        res = json.loads(txn.query(query, variables=variables).json)
        if not res['q0']:
            return []
        return res['q0'][0]['scope']
    finally:
        txn.discard()

def expand_forward_edges_in_scope(dgraph_client: DgraphClient, node: NodeView, lens: str) -> None:
    for edge_name, edge_type in node._get_forward_edge_types().items():

        if isinstance(edge_type, list):
            inner_edge_type = edge_type[0]
        else:
            inner_edge_type = edge_type
        edges_in_lens = edge_in_lens(dgraph_client, node.uid, edge_name, lens)
        for edge in edges_in_lens:
            for neighbors in edge.values():
                if not isinstance(neighbors, list):
                    neighbors = [neighbors]
                for neighbor in neighbors:
                    if neighbor.get('~scope'):
                        neighbor.pop('~scope')
                    node_edge = getattr(node, edge_name)
                    print(edge_name)
                    neighbor_view = inner_edge_type(dgraph_client, node_key=neighbor['node_key'], uid=neighbor['uid'])
                    print(neighbor_view, neighbor_view.uid, neighbor_view.node_key)
                    if isinstance(node_edge, list):
                        node_edge.append(neighbor_view)
                    else:
                        node_edge = neighbor_view
                        setattr(node, edge_name, node_edge)


def expand_reverse_edges_in_scope(dgraph_client: DgraphClient, node: NodeView, lens: str) -> None:
    for edge_name, (edge_type, forward_name) in node._get_reverse_edge_types().items():

        if isinstance(edge_type, list):
            inner_edge_type = edge_type[0]
        else:
            inner_edge_type = edge_type
        edges_in_lens = edge_in_lens(dgraph_client, node.uid, edge_name, lens)
        for edge in edges_in_lens:
            for neighbors in edge.values():

                if not isinstance(neighbors, list):
                    neighbors = [neighbors]

                for neighbor in neighbors:
                    if neighbor.get('~scope'):
                        neighbor.pop('~scope')
                    neighbor_view = inner_edge_type(
                        dgraph_client,
                        node_key=neighbor['node_key'],
                        uid=neighbor['uid'],
                    )

                    node_edge = getattr(node, forward_name)

                    if isinstance(node_edge, list):
                        node_edge.append(neighbor_view)
                    else:
                        node_edge = neighbor_view
                        setattr(node, forward_name, node_edge)


def expand_concrete_nodes(dgraph_client: DgraphClient, lens_name: str,
                          concrete_nodes: List[NodeView]) -> None:

    for node in concrete_nodes:
        expand_forward_edges_in_scope(dgraph_client, node, lens_name)
        expand_reverse_edges_in_scope(dgraph_client, node, lens_name)

    for node in concrete_nodes:
        for prop_name, prop_type in node._get_property_types().items():
            setattr(node, prop_name, node.fetch_property(prop_name, prop_type))


def expand_node_forward(dgraph_client: DgraphClient, node_key: str) -> Optional[Dict[str, Any]]:
    query = """
        query res($node_key: string)
        {
        
            res(func: eq(node_key, $node_key))
            {
                uid,
                expand(_forward_) {
                    uid,
                    expand(_forward_),
                    node_type: dgraph.type
                }
                node_type: dgraph.type
            }
      
        }
    """

    txn = dgraph_client.txn(read_only=True)
    variables = {"$node_key": node_key}
    try:
        res = json.loads(txn.query(query, variables=variables).json)
    finally:
        txn.discard()
    return res['res'][0]


def expand_dynamic_node(dynamic_node: DynamicNodeView) -> Dict[str, Any]:
    node = raw_node_from_node_key(dynamic_node.dgraph_client, dynamic_node.node_key)
    edges = []
    expanded_node = expand_node_forward(dynamic_node.dgraph_client, dynamic_node.node_key)
    assert expanded_node, 'expanded_node'
    for prop, val in expanded_node.items():
        if prop == 'node_type' or prop == "dgraph.type" or prop == "risks":
            continue

        if isinstance(val, list):
            if val and isinstance(val[0], dict):
                for edge in val:
                    edges.append({
                        "from": dynamic_node.node_key,
                        "edge_name": prop,
                        "to": edge["node_key"]
                    })
        if isinstance(val, dict):
            edges.append({
                "from": dynamic_node.node_key,
                "edge_name": prop,
                "to": val["node_key"]
            })

    return {
        "node": node,
        "edges": edges
    }


def lens_to_dict(dgraph_client: DgraphClient, lens_name: str) -> List[Dict[str, Any]]:
    current_graph = get_lens_scope(dgraph_client, lens_name)
    print('Getting len sas dict')
    if not current_graph:
        return []
    nodes = []
    for graph in current_graph['scope']:
        nodes.append(NodeView.from_dict(dgraph_client, graph))

    if current_graph.get('scope'):
        current_graph.pop('scope')

    concrete_nodes = [n.node for n in nodes if not isinstance(n.node, _DynamicNodeView)]
    dynamic_nodes = [n.node for n in nodes if isinstance(n.node, _DynamicNodeView)]

    expanded_dynamic_nodes = []
    for dynamic_node in dynamic_nodes:
        print('aaa')
        expanded = expand_dynamic_node(dynamic_node)
        expanded_dynamic_nodes.append(expanded)

    expand_concrete_nodes(
        dgraph_client,
        lens_name,
        concrete_nodes
    )

    results = [{
        "node": current_graph,
        "edges": []
    }]

    lens_risks = get_lens_risks(dgraph_client, lens_name)
    for node in lens_risks:
        edges = []
        risks = node.get("risks", [])
        if not risks:
            print(f"WARN: Node in engagement graph has no connected risks {node['node_key']}")
        for risk in risks:
            risk['node_key'] = node['node_key'] + risk['analyzer_name']
            edge = {
                "from": node["node_key"],
                "edge_name": "risks",
                "to": risk['node_key']
            }
            edges.append(edge)

        results.append({
            "node": node,
            "edges": edges,
        })

    results.extend([n.to_dict() for n in concrete_nodes])
    results.extend(expanded_dynamic_nodes)
    return results

def try_get_updated_graph(body):
    print('Trying to update graph')
    client_stub = pydgraph.DgraphClientStub('alpha0.engagementgraphcluster.grapl:9080')
    dg_client = pydgraph.DgraphClient(client_stub)

    lens = body["lens"]

    # Mapping from `uid` to node hash
    initial_graph = body["uid_hashes"]

    # print(f'lens: {lens} initial_graph: {initial_graph}')
    #
    # # Try for 20 seconds max
    # max_time = int(time.time()) + 20
    while True:
        print("Getting updated graph")
        current_graph = lens_to_dict(dg_client, lens)

        updates = {
            'updated_nodes': current_graph,
            'removed_nodes': []
        }

        return updates


def respond(err, res=None, headers=None, origin_override=None):
    if not headers:
        headers = {}

    return {
        'statusCode': '400' if err else '200',
        'body': {'error': err} if err else json.dumps({'success': res}),
        'headers': {
            'Access-Control-Allow-Origin': origin_override or ORIGIN,
            'Access-Control-Allow-Credentials': True,
            'Content-Type': 'application/json',
            'Access-Control-Allow-Methods': 'GET,POST,OPTIONS',
            'X-Requested-With': '*',
            **headers
        },
    }


def get_salt_and_pw(table, username):
    print(f'Getting salt for user: {username}')
    response = table.get_item(
        Key={
            'username': username,
        }
    )

    if not response.get('Item'):
        return None, None

    return response['Item']['salt'].value, response['Item']['password'].value


def hash_password(cleartext, salt) -> str:
    print('initial hash')
    hashed = sha256(cleartext).digest()

    hasher = blake2b(salt=salt)
    hasher.update(hashed)
    return hasher.digest()


def user_auth_table():
    global DYNAMO
    DYNAMO = DYNAMO or boto3.resource('dynamodb')

    return DYNAMO.Table(os.environ['USER_AUTH_TABLE'])


def create_user(username, cleartext):
    table = user_auth_table()
    # We hash before calling 'hashed_password' because the frontend will also perform
    # client side hashing
    pepper = "f1dafbdcab924862a198deaa5b6bae29aef7f2a442f841da975f1c515529d254";

    hashed = sha256(cleartext + pepper).digest()
    for i in range(0, 5000):
        hashed = sha256(hashed).digest()

    salt = os.urandom(blake2b.SALT_SIZE)
    password = hash_password(hashed, salt)

    table.put_item(
        Item={
            'username': username,
            'salt': salt,
            'password': password
        }
    )


def login(username, password):
    # Connect to dynamodb table
    table = user_auth_table()

    # Get salt for username
    salt, true_pw = get_salt_and_pw(table, username)
    if not salt or not true_pw:
        return None

    # Hash password
    to_check = hash_password(password.encode('utf8'), salt)
    print('hashed')

    if not compare_digest(to_check, true_pw):
        time.sleep(round(uniform(0.1, 3.0), 2))
        return None

    # Use JWT to generate token
    return jwt.encode({'username': username}, JWT_SECRET, algorithm='HS256').decode('utf8')


def check_jwt(headers):
    encoded_jwt = None
    print(f'headers: {headers}')
    for cookie in headers.get('Cookie', '').split(';'):
        if 'grapl_jwt=' in cookie:
            encoded_jwt = cookie.split('grapl_jwt=')[1].strip()

    if not encoded_jwt:
        return False

    try:
        jwt.decode(encoded_jwt, JWT_SECRET, algorithms=['HS256'])
        return True
    except Exception as e:
        print(e)
        return False


def lambda_login(event):
    body = json.loads(event['body'])
    print(f'body: {body}')
    login_res = login(body['username'], body['password'])
    # Clear out the password from the dict, to avoid accidentally logging it
    body['password'] = ''
    cookie = f"grapl_jwt={login_res}; secure; HttpOnly; SameSite=None"
    if login_res:
        return cookie


def lambda_handler(event, context):
    try:
        if event['httpMethod'] == 'OPTIONS':
            return respond(None, {})

        if '/login' in event['path']:
            cookie = lambda_login(event)
            if cookie:
                return respond(None, 'True', headers={'Set-Cookie': cookie})
            else:
                return respond('Failed to login')

        if '/checkLogin' in event['path']:
            print('logging in')
            if check_jwt(event['headers']):
                return respond(None, 'True')
            else:
                return respond(None, 'False')

        if not check_jwt(event['headers']):
            return respond("Must log in")

        if '/update' in event['path']:
            update = try_get_updated_graph(json.loads(event["body"]))
            return respond(None, update)

        if '/getLenses' in event['path']:
            prefix = json.loads(event["body"]).get('prefix', '')
            lenses = list_all_lenses(prefix)
            return respond(None, {'lenses': lenses})

        return respond(f"Invalid path: {event['path']}", {})
    except Exception as e:
        traceback.print_exc()
        return respond("UnknownError")
