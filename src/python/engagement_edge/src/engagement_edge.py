import json
import logging
import os
import sys
import time
import uuid

from hashlib import sha256, pbkdf2_hmac
from hmac import compare_digest
from random import uniform
from typing import List, Dict, Any, Optional

import boto3
import jwt
import pydgraph
from chalice import Chalice, Response, CORSConfig

from grapl_analyzerlib.nodes.any_node import NodeView, raw_node_from_node_key
from grapl_analyzerlib.nodes.dynamic_node import DynamicNodeView
from pydgraph import DgraphClient

IS_LOCAL = bool(os.environ.get("IS_LOCAL", False))

if IS_LOCAL:
    import time
    while True:
        try:
            secretsmanager = boto3.client(
                "secretsmanager",
                region_name='us-east-1',
                aws_access_key_id="dummy_cred_aws_access_key_id",
                aws_secret_access_key="dummy_cred_aws_secret_access_key",
                endpoint_url='http://secretsmanager.us-east-1.amazonaws.com:4566'
            )

            JWT_SECRET = secretsmanager.get_secret_value(SecretId='JWT_SECRET_ID',)["SecretString"]
            print(JWT_SECRET)
            break
        except Exception as e:
            print(e)
            time.sleep(1)

    print("JWT SECRET", JWT_SECRET)
else:
    JWT_SECRET_ID = os.environ["JWT_SECRET_ID"]

    secretsmanager = boto3.client("secretsmanager")

    JWT_SECRET = secretsmanager.get_secret_value(SecretId=JWT_SECRET_ID,)["SecretString"]

ORIGIN = os.environ["UX_BUCKET_URL"].lower()

ORIGIN_OVERRIDE = os.environ.get("ORIGIN_OVERRIDE", None)
DYNAMO = None

if IS_LOCAL:
    MG_ALPHA = "master_graph:9080"
else:
    MG_ALPHA = "alpha0.mastergraphcluster.grapl:9080"

GRAPL_LOG_LEVEL = os.getenv("GRAPL_LOG_LEVEL")
LEVEL = "ERROR" if GRAPL_LOG_LEVEL is None else GRAPL_LOG_LEVEL
LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(LEVEL)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

app = Chalice(app_name="engagement-edge")


def list_all_lenses(prefix: str) -> List[Dict[str, Any]]:
    LOGGER.info(f"connecting to dgraph at {MG_ALPHA}")
    client_stub = pydgraph.DgraphClientStub(MG_ALPHA)
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
                    node_type: dgraph.type,
                    lens,
                    score
                }
            }"""

        variables = {"$a": prefix}
    else:
        query = """
            {
                q0(func: has(lens), orderdesc: score)
                {
                    uid,
                    node_key,
                    node_type: dgraph.type,
                    lens,
                    score
                }
            }"""

        variables = {}

    txn = dg_client.txn(read_only=True)

    try:
        res = json.loads(txn.query(query, variables=variables).json)
        return res["q0"]
    finally:
        txn.discard()


def edge_in_lens(
    dg_client: DgraphClient, node_uid: str, edge_name: str, lens_name: str
) -> List[Dict[str, Any]]:
    query = f"""
        query q0($node_uid: string, $lens_name: string)
        {{
            q0(func: uid($node_uid)) @cascade {{
                {edge_name} {{
                    uid,
                    node_key,
                    node_type: dgraph.type,

                    ~scope @filter(eq(lens, $lens_name)) {{
                        uid,
                        node_type: dgraph.type,
                    }}
                }}
            }}
        }}
    """

    txn = dg_client.txn(read_only=True)

    try:
        variables = {"$node_uid": node_uid, "$lens_name": lens_name}
        res = json.loads(txn.query(query, variables=variables).json)
        return res["q0"]
    finally:
        txn.discard()


def get_lens_scope(dg_client: DgraphClient, lens: str) -> Dict[str, Any]:
    query = """
        query q0($a: string)
        {
            q0(func: eq(lens, $a)) {
                uid,
                node_type: dgraph.type,
                node_key,
                lens,
                score,
                scope {
                    uid,
                    expand(_all_),
                    node_type: dgraph.type,
                }
            }
      }"""

    txn = dg_client.txn(read_only=True)

    try:
        variables = {"$a": lens}
        res = json.loads(txn.query(query, variables=variables).json)
        if not res["q0"]:
            return {}
        return res["q0"][0]
    finally:
        txn.discard()


def get_lens_risks(dg_client: DgraphClient, lens: str) -> List[Dict[str, Any]]:
    query = """
        query q0($a: string)
        {
            q0(func: eq(lens, $a)) {
                uid,
                node_type: dgraph.type,
                node_key,
                lens,
                score,
                scope {
                    uid,
                    node_key,
                    node_type: dgraph.type
                    risks {
                        uid,
                        node_key,
                        analyzer_name,
                        node_type: dgraph.type,
                        risk_score
                    }
                }
            }
      }"""

    txn = dg_client.txn(read_only=True)

    try:
        variables = {"$a": lens}
        res = json.loads(txn.query(query, variables=variables).json)
        if not res["q0"]:
            return []
        return res["q0"][0]["scope"]
    finally:
        txn.discard()


def expand_forward_edges_in_scope(
    dgraph_client: DgraphClient, node: NodeView, lens: str
) -> None:
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
                    if neighbor.get("~scope"):
                        neighbor.pop("~scope")
                    node_edge = getattr(node, edge_name)
                    try:
                        neighbor_view = inner_edge_type(
                            dgraph_client,
                            node_key=neighbor["node_key"],
                            uid=neighbor["uid"],
                        )
                    except Exception as e:
                        LOGGER.error(f"neighbor_view failed with: {e}")
                        continue
                    LOGGER.debug(
                        neighbor_view, neighbor_view.uid, neighbor_view.node_key
                    )
                    if isinstance(node_edge, list):
                        node_edge.append(neighbor_view)
                    else:
                        node_edge = neighbor_view
                        setattr(node, edge_name, node_edge)


def expand_reverse_edges_in_scope(
    dgraph_client: DgraphClient, node: NodeView, lens: str
) -> None:
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
                    if neighbor.get("~scope"):
                        neighbor.pop("~scope")
                    neighbor_view = inner_edge_type(
                        dgraph_client,
                        node_key=neighbor["node_key"],
                        uid=neighbor["uid"],
                    )

                    node_edge = getattr(node, forward_name)

                    if isinstance(node_edge, list):
                        node_edge.append(neighbor_view)
                    else:
                        node_edge = neighbor_view
                        setattr(node, forward_name, node_edge)


def expand_concrete_nodes(
    dgraph_client: DgraphClient, lens_name: str, concrete_nodes: List[NodeView]
) -> None:
    for node in concrete_nodes:
        expand_forward_edges_in_scope(dgraph_client, node, lens_name)
        expand_reverse_edges_in_scope(dgraph_client, node, lens_name)

    for node in concrete_nodes:
        for prop_name, prop_type in node._get_property_types().items():
            setattr(node, prop_name, node.fetch_property(prop_name, prop_type))


def expand_node_forward(
    dgraph_client: DgraphClient, node_key: str
) -> Optional[Dict[str, Any]]:
    query = """
        query res($node_key: string)
        {

            res(func: eq(node_key, $node_key))
            {
                uid,
                expand(_all_) {
                    uid,
                    expand(_all_),
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
    return res["res"][0]


def expand_dynamic_node(dynamic_node: DynamicNodeView) -> Dict[str, Any]:
    node = raw_node_from_node_key(dynamic_node.dgraph_client, dynamic_node.node_key)
    edges = []
    expanded_node = expand_node_forward(
        dynamic_node.dgraph_client, dynamic_node.node_key
    )
    assert expanded_node, "expanded_node"
    for prop, val in expanded_node.items():
        if prop == "node_type" or prop == "dgraph.type" or prop == "risks":
            continue

        if isinstance(val, list):
            if val and isinstance(val[0], dict):
                for edge in val:
                    edges.append(
                        {
                            "from": dynamic_node.node_key,
                            "edge_name": prop,
                            "to": edge["node_key"],
                        }
                    )
        if isinstance(val, dict):
            edges.append(
                {
                    "from": dynamic_node.node_key,
                    "edge_name": prop,
                    "to": val["node_key"],
                }
            )

    return {"node": node, "edges": edges}


def lens_to_dict(dgraph_client: DgraphClient, lens_name: str) -> List[Dict[str, Any]]:
    current_graph = get_lens_scope(dgraph_client, lens_name)
    LOGGER.info(f"Getting lens as dict {current_graph}")
    if not current_graph or not current_graph.get("scope"):
        return []
    nodes = []
    for graph in current_graph["scope"]:
        try:
            nodes.append(NodeView.from_dict(dgraph_client, graph))
        except Exception as e:
            LOGGER.error("Failed to get NodeView from dict", e)
    if current_graph.get("scope"):
        current_graph.pop("scope")

    concrete_nodes = [n.node for n in nodes if not isinstance(n.node, DynamicNodeView)]
    dynamic_nodes = [n.node for n in nodes if isinstance(n.node, DynamicNodeView)]

    expanded_dynamic_nodes = []
    for dynamic_node in dynamic_nodes:
        expanded = expand_dynamic_node(dynamic_node)
        expanded_dynamic_nodes.append(expanded)

    expand_concrete_nodes(dgraph_client, lens_name, concrete_nodes)

    results = [{"node": current_graph, "edges": []}]

    lens_risks = get_lens_risks(dgraph_client, lens_name)
    for node in lens_risks:
        edges = []
        risks = node.get("risks", [])
        if not risks:
            LOGGER.warning(f"Node in engagement graph has no connected risks {node}")
        for risk in risks:
            try:
                risk["node_key"] = node["node_key"] + risk["analyzer_name"]
                edge = {
                    "from": node["node_key"],
                    "edge_name": "risks",
                    "to": risk["node_key"],
                }
                edges.append(edge)
            except Exception as e:
                LOGGER.error(f"risk edge failed: {risk} {e}")

        results.append(
            {"node": node, "edges": edges,}
        )

    results.extend([n.to_dict() for n in concrete_nodes])
    results.extend(expanded_dynamic_nodes)
    return results


def try_get_updated_graph(body):
    LOGGER.info("Trying to update graph")
    LOGGER.info(f"connecting to dgraph at {MG_ALPHA}")
    client_stub = pydgraph.DgraphClientStub(MG_ALPHA)
    dg_client = pydgraph.DgraphClient(client_stub)

    lens = body["lens"]

    # Mapping from `uid` to node hash
    initial_graph = body["uid_hashes"]

    while True:
        LOGGER.info("Getting updated graph")
        current_graph = lens_to_dict(dg_client, lens)

        updates = {"updated_nodes": current_graph, "removed_nodes": []}

        return updates


def respond(err, res=None, headers=None):
    LOGGER.info(f"responding, origin: {app.current_request.headers.get('origin', '')}")
    if not headers:
        headers = {}

    if IS_LOCAL:
        override = app.current_request.headers.get("origin", "")
        LOGGER.info(f"overriding origin with {override}")
    else:
        override = ORIGIN_OVERRIDE

    return Response(
        body={"error": err} if err else json.dumps({"success": res}),
        status_code=400 if err else 200,
        headers={
            "Access-Control-Allow-Origin": override or ORIGIN,
            "Access-Control-Allow-Credentials": "true",
            "Content-Type": "application/json",
            "Access-Control-Allow-Methods": "GET,POST,OPTIONS",
            "X-Requested-With": "*",
            "Access-Control-Allow-Headers": "Content-Type, Access-Control-Allow-Headers, Authorization, X-Requested-With",
            **headers,
        },
    )


def get_salt_and_pw(table, username):
    LOGGER.info(f"Getting salt for user: {username}")
    response = table.get_item(Key={"username": username,})

    if not response.get("Item"):
        return None, None

    salt = response["Item"]["salt"].value
    password = response["Item"]["password"]
    return salt, password


def hash_password(cleartext, salt) -> str:
    hashed = sha256(cleartext).digest()
    return pbkdf2_hmac("sha256", hashed, salt, 512000).hex()


def user_auth_table():
    global DYNAMO
    DYNAMO = DYNAMO or boto3.resource("dynamodb")

    return DYNAMO.Table(os.environ["USER_AUTH_TABLE"])


def create_user(username, cleartext):
    table = user_auth_table()
    # We hash before calling 'hashed_password' because the frontend will also perform
    # client side hashing
    pepper = "f1dafbdcab924862a198deaa5b6bae29aef7f2a442f841da975f1c515529d254"

    hashed = sha256(cleartext + pepper + username).digest()
    for i in range(0, 5000):
        hashed = sha256(hashed).digest()

    salt = os.urandom(16)
    password = hash_password(hashed, salt)

    table.put_item(Item={"username": username, "salt": salt, "password": password})


def login(username, password):
    if IS_LOCAL:
        return jwt.encode({"username": username}, JWT_SECRET, algorithm="HS256").decode(
            "utf8"
        )
    # Connect to dynamodb table
    table = user_auth_table()

    # Get salt for username
    salt, true_pw = get_salt_and_pw(table, username)
    if not salt or not true_pw:
        return None

    # Hash password
    to_check = hash_password(password.encode("utf8"), salt)
    LOGGER.debug("hashed")

    if not compare_digest(to_check, true_pw):
        time.sleep(round(uniform(0.1, 3.0), 2))
        return None

    # Use JWT to generate token
    return jwt.encode({"username": username}, JWT_SECRET, algorithm="HS256").decode(
        "utf8"
    )


def check_jwt(headers):
    # if IS_LOCAL:
    #     return True

    encoded_jwt = None
    for cookie in headers.get("Cookie", "").split(";"):
        if "grapl_jwt=" in cookie:
            encoded_jwt = cookie.split("grapl_jwt=")[1].strip()

    if not encoded_jwt:
        return False

    try:
        jwt.decode(encoded_jwt, JWT_SECRET, algorithms=["HS256"])
        return True
    except Exception as e:
        LOGGER.error(e)
        return False


def lambda_login(event):
    body = event.json_body
    login_res = login(body["username"], body["password"])
    # Clear out the password from the dict, to avoid accidentally logging it
    body["password"] = ""
    cookie = (
        f"grapl_jwt={login_res}; Domain=.amazonaws.com; secure; HttpOnly; SameSite=None"
    )
    if login_res:
        return cookie


cors_config = CORSConfig(
    allow_origin=ORIGIN_OVERRIDE or ORIGIN, allow_credentials="true",
)


def requires_auth(path):
    if not IS_LOCAL:
        path = "/{proxy+}" + path

    def route_wrapper(route_fn):
        @app.route(path, methods=["OPTIONS", "POST"])
        def inner_route():
            if app.current_request.method == "OPTIONS":
                return respond(None, {})

            # if not IS_LOCAL:  # For now, disable authentication locally
            if not check_jwt(app.current_request.headers):
                LOGGER.warn("not logged in")
                return respond("Must log in")
            try:
                return route_fn()
            except Exception as e:
                LOGGER.error(e)
                return respond("Unexpected Error")

        return inner_route

    return route_wrapper


def no_auth(path):
    if not IS_LOCAL:
        path = "/{proxy+}" + path

    def route_wrapper(route_fn):
        @app.route(path, methods=["OPTIONS", "GET", "POST"])
        def inner_route():
            if app.current_request.method == "OPTIONS":
                return respond(None, {})
            try:
                return route_fn()
            except Exception as e:
                LOGGER.error(e)
                return respond("Unexpected Error")

        return inner_route

    return route_wrapper


@no_auth("/login")
def login_route():
    LOGGER.debug("/login_route")
    request = app.current_request
    cookie = lambda_login(request)
    if cookie:
        LOGGER.info("logged in")
        return respond(None, "True", headers={"Set-Cookie": cookie})
    else:
        LOGGER.warn("not logged in")
        return respond("Failed to login")


@no_auth("/checkLogin")
def check_login():
    LOGGER.debug("/checkLogin")
    request = app.current_request
    if check_jwt(request.headers):
        return respond(None, "True")
    else:
        return respond(None, "False")


@app.route("/{proxy+}", methods=["OPTIONS", "POST", "GET"])
def nop_route():
    LOGGER.debug(app.current_request.context["path"])

    path = app.current_request.context["path"]

    if path == "/prod/login":
        return login_route()
    elif path == "/prod/checkLogin":
        return check_login()

    return respond("InvalidPath")
