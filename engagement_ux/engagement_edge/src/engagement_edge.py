from typing import List, Dict, Any

from pydgraph import DgraphClient

print('import boto3')
import boto3

print('import pydgraph')
import pydgraph

print('import json')
import json

print('import time')
import time

from hashlib import sha256


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


def get_lens_scope(dg_client, lens):
    query = """
        query q0($a: string)
        {
            q as var(func: eq(lens, $a)) {
               
            }
        
            q0(func: uid(q)) {
                uid,
                node_key,
                lens,
                score,
                scope {
                  uid,
                  expand(_all_)
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
    if node.get('process_id'):
        for prop in process_properties:
            hash_str += str(node.get(prop, ""))

    if node.get('file_path'):
        for prop in file_properties:
            hash_str += str(node.get(prop, ""))

    for edge in edge_names:
        if node.get(edge):
            hash_str += edge + str(len(node[edge]))
        else:
            hash_str += edge + '0'

    # return hash_str
    return sha256(hash_str.encode()).hexdigest()


def get_updated_graph(dg_client, initial_graph, lens):
    current_graph = get_lens_scope(dg_client, lens)

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


def respond(err, res=None):
    return {
        'statusCode': '400' if err else '200',
        'body': err if err else json.dumps(res),
        'headers': {
            'Access-Control-Allow-Origin': '*',
            'Content-Type': 'application/json',
            'Access-Control-Allow-Methods': 'GET,POST,OPTIONS',
            'X-Requested-With': '*',
        },
    }


def lambda_handler(event, context):
    try:
        print(f"httpMethod: {event['httpMethod']}")
        print(f"path: {event['path']}")

        if event['httpMethod'] == 'OPTIONS':
            return respond(None, {})

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
        return respond("Error fetching updates {}".format(e))

