import os
from typing import List, Dict, Any

from random import uniform
from hashlib import sha256, blake2b
from hmac import compare_digest

import boto3
from pydgraph import DgraphClient
import jwt
import pydgraph
import json
import time

JWT_SECRET = os.environ['JWT_SECRET']
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

# Just query the schema in the future
process_properties = [
    'process_id', 'node_key', 'create_time', 'arguments',
    'process_name'
]

file_properties = [
    'node_key', 'file_path'
]


edge_names = [
    'children',
    'bin_file',
    'created_file',
    'scope',
]

# Get all nodes in a lens scope, and all of the edges from nodes in the scope to other nodes in the scope
def get_lens_scope(dg_client, lens):
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
                    expand(_forward_) {
                        uid,    
                        node_key,
                        process_name,
                        process_id,
                        file_path,
                        node_type,
                        port,
                        created_timestamp,
                        analyzer_name,
                        risk_score,
                        ~scope @filter(eq(lens, $a) OR has(risk_score)) {
                            uid, node_key, analyzer_name, risk_score,
                            lens, score
                        }
                    }
                }
            }  
      }"""

    txn = dg_client.txn(read_only=True)

    try:
        variables = {'$a': lens}
        res = json.loads(txn.query(query, variables=variables).json)
        return res['q0']
    finally:
        txn.discard()


def hash_node(node):
    hash_str = str(node['uid'])
    print(node)
    props = []
    for prop_name, prop_value in node:
        if isinstance(prop_value, list):
            if len(prop_value) > 0 and isinstance(prop_value[0], dict):
                if prop_value[0].get('uid'):
                    continue

        props.append(prop_name + str(prop_value))

    props.sort()
    hash_str += "".join(props)

    edges = []

    for prop_name, prop_value in node:
        if isinstance(prop_value, list):
            if len(prop_value) > 0 and isinstance(prop_value[0], dict):
                if not prop_value[0].get('uid'):
                    continue
                edge_uids = []
                for edge in prop_value:
                    edges.append(prop_name + edge['uid'])

                edge_uids.sort()
                edges.append("".join(edge_uids))

    edges.sort()
    print(edges)
    hash_str += "".join(edges)
    # return hash_str
    return sha256(hash_str.encode()).hexdigest()


def strip_graph(graph, lens, edgename='scope'):
    for outer_node in graph.get(edgename, []):
        for prop, val in outer_node.items():
            if prop == 'risks' or prop == '~risks':
                continue

            if isinstance(val, list) and isinstance(val[0], dict):
                new_vals = []
                for inner_val in val:
                    rev_scope = inner_val.get('~scope', [])
                    to_keep = False
                    for n in rev_scope:
                        if (n.get('lens') == lens) or n.get('analyzer_name'):
                            to_keep = True
                    if to_keep:
                        new_vals.append(inner_val)
                outer_node[prop] = new_vals


def get_updated_graph(dg_client, initial_graph, lens):
    current_graph = get_lens_scope(dg_client, lens)
    for graph in current_graph:
        strip_graph(graph, lens)

    new_or_modified = []
    for node in current_graph:
        if initial_graph.get(node['uid']):
            node_hash = initial_graph[node['uid']]
            if node_hash != hash_node(node):
                new_or_modified.append(node)
        else:
            new_or_modified.append(node)

    all_uids = []
    for node in current_graph:
        if node.get('scope'):
            all_uids.extend([node['uid'] for node in node.get('scope')])
        all_uids.append(node['uid'])

    removed_uids = set(initial_graph.keys()) - \
                   set(all_uids)

    return new_or_modified, list(removed_uids)


def try_get_updated_graph(body):
    print('Trying to update graph')
    client_stub = pydgraph.DgraphClientStub('alpha0.engagementgraphcluster.grapl:9080')
    dg_client = pydgraph.DgraphClient(client_stub)

    lens = body["lens"]

    # Mapping from `uid` to node hash
    initial_graph = body["uid_hashes"]

    print(f'lens: {lens} initial_graph: {initial_graph}')

    # Try for 20 seconds max
    max_time = int(time.time()) + 20
    while True:
        print("Getting updated graph")
        updated_nodes, removed_nodes = get_updated_graph(
            dg_client,
            initial_graph,
            lens
        )

        updates = {
            'updated_nodes': updated_nodes,
            'removed_nodes': removed_nodes
        }

        if updated_nodes or removed_nodes:
            print("Graph has been updated: ")
            return updates

        now = int(time.time())

        if now >= max_time:
            print("Timed out before finding an update")
            return updates
        print("Graph has not updated")
        time.sleep(0.75)


def respond(err, res=None, headers=None):
    if not headers:
        headers = {}
    return {
        'statusCode': '400' if err else '200',
        'body': {'error': err} if err else json.dumps({'success': res}),
        'headers': {
            'Access-Control-Allow-Origin': '*',
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

    print(f'hashing password {password}')
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
    print(f'login_res: {login_res}')
    if login_res:
        return respond(None, 'True', headers={'Set-Cookie': 'grapl_jwt=' + login_res})
    else:
        return respond('Invalid user or password')


def lambda_handler(event, context):

    try:
        if event['httpMethod'] == 'OPTIONS':
            return respond(None, {})

        if '/login' in event['path']:
            return lambda_login(event)

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
        print('Failed with e {}'.format(e))
        return respond("UnknownError")

