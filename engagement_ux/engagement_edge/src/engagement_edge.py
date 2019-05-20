print('import boto3')
import boto3

print('import pydgraph')
import pydgraph

print('import json')
import json

print('import time')
import time

from hashlib import sha256


# Just query the schema in the future
process_properties = [
    'pid', 'node_key', 'create_time', 'arguments',
    'image_name'
]

file_properties = [
    'pid', 'node_key', 'path'
]


edge_names = [
    'children',
    'bin_file',
    'created_file',
]


def get_engagement_graph(dg_client, eg_id):
    query = """
        query q0($a: string)
        {
            q0(func: eq(engagement_id, $a)) {
                uid,
                _predicate_,
                expand(_forward_) {
                  uid,
                  node_key,
                  _predicate_
                }
            }
        }"""

    txn = dg_client.txn(read_only=True)

    try:
        variables = {'$a': eg_id}
        res = json.loads(txn.query(query, variables=variables).json)
        return res['q0']
    finally:
        txn.discard()


def hash_node(node):
    hash_str = str(node['uid'])
    print(node)
    if node.get('pid'):
        for prop in process_properties:
            hash_str += str(node.get(prop, ""))

    if node.get('path'):
        for prop in file_properties:
            hash_str += str(node.get(prop, ""))

    for edge in edge_names:
        if node.get(edge):
            hash_str += edge + str(len(node[edge]))
        else:
            hash_str += edge + '0'

    # return hash_str
    return sha256(hash_str).hexdigest()


def get_updated_graph(dg_client, initial_graph, engagement_id):
    current_graph = get_engagement_graph(dg_client, engagement_id)

    new_or_modified = []
    for node in current_graph:
        if initial_graph.get(node['uid']):
            node_hash = initial_graph[node['uid']]
            print("hashes")
            print(hash_node(node))
            print(node_hash)
            print("post-hashes")
            if node_hash != hash_node(node):
                new_or_modified.append(node)
        else:
            new_or_modified.append(node)

    removed_uids = set(initial_graph.keys()) - \
                   set([node['uid'] for node in current_graph])

    return new_or_modified, list(removed_uids)


def try_get_updated_graph(body):
    print('Tring to update graph')
    client_stub = pydgraph.DgraphClientStub('alpha1.engagementgraphcluster.grapl:9080')
    dg_client = pydgraph.DgraphClient(client_stub)

    engagement_id = body["engagement_id"]

    # Mapping from `uid` to node hash
    initial_graph = body["uid_hashes"]

    # Try for 20 seconds max
    max_time = int(time.time()) + 20
    while True:
        print("Getting updated graph")
        updated_nodes, removed_nodes = get_updated_graph(
            dg_client,
            initial_graph,
            engagement_id
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
            'Content-Type': 'application/json',
        },
    }


def lambda_handler(event, context):
    print("Received event: " + json.dumps(event, indent=2))

    try:
        update = try_get_updated_graph(event["body"])
    except Exception as e:
        print('Failed with e {}'.format(e))
        return respond("Error fetching updates {}".format(e))

    return respond(None, update)