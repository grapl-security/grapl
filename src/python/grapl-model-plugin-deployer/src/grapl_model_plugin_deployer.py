import base64
import hmac
import inspect
import json
import logging
import os
import re
import sys
import threading
import traceback
from base64 import b64decode
from hashlib import sha1
from http import HTTPStatus
from pathlib import Path
from typing import Any, Dict, List, Optional, TypeVar, Union

import boto3  # type: ignore
import jwt
import pydgraph  # type: ignore
from botocore.client import BaseClient  # type: ignore
from chalice import Chalice, Response
from github import Github
from grapl_common.env_helpers import S3ClientFactory

from grapl_analyzerlib.node_types import (
    EdgeRelationship,
    EdgeT,
    PropPrimitive,
    PropType,
)
from grapl_analyzerlib.prelude import *
from grapl_analyzerlib.schema import Schema

sys.path.append("/tmp/")

T = TypeVar("T")

IS_LOCAL = bool(os.environ.get("IS_LOCAL", False))

GRAPL_LOG_LEVEL = os.getenv("GRAPL_LOG_LEVEL")
LEVEL = "ERROR" if GRAPL_LOG_LEVEL is None else GRAPL_LOG_LEVEL
LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(LEVEL)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))
LOGGER.info("Initializing Chalice server")

try:
    directory = Path("/tmp/model_plugins/")
    directory.mkdir(parents=True, exist_ok=True)
except Exception as e:
    LOGGER.error("Failed to create directory", e)

if IS_LOCAL:
    import time

    for i in range(0, 150):
        try:
            secretsmanager = boto3.client(
                "secretsmanager",
                region_name="us-east-1",
                aws_access_key_id="dummy_cred_aws_access_key_id",
                aws_secret_access_key="dummy_cred_aws_secret_access_key",
                endpoint_url="http://secretsmanager.us-east-1.amazonaws.com:4584",
            )

            JWT_SECRET = secretsmanager.get_secret_value(
                SecretId="JWT_SECRET_ID",
            )["SecretString"]
            break
        except Exception as e:
            LOGGER.debug(e)
            time.sleep(1)

    os.environ["BUCKET_PREFIX"] = "local-grapl"
else:
    JWT_SECRET_ID = os.environ["JWT_SECRET_ID"]

    client = boto3.client("secretsmanager")

    JWT_SECRET = client.get_secret_value(
        SecretId=JWT_SECRET_ID,
    )["SecretString"]

app = Chalice(app_name="model-plugin-deployer")


def into_list(t: Union[T, List[T]]) -> List[T]:
    if isinstance(t, list):
        return t
    return [t]


def check_jwt(headers):
    encoded_jwt = None
    for cookie in headers.get("Cookie", "").split(";"):
        if "grapl_jwt=" in cookie:
            encoded_jwt = cookie.split("grapl_jwt=")[1].strip()

    if not encoded_jwt:
        return False

    try:
        jwt.decode(encoded_jwt, JWT_SECRET, algorithms=["HS256"])
        return True
    except Exception:
        LOGGER.error(traceback.format_exc())
        return False


def verify_payload(payload_body, key, signature):
    new_signature = "sha1=" + hmac.new(key, payload_body, sha1).hexdigest()
    return new_signature == signature


def set_schema(client: GraphClient, schema: str) -> None:
    op = pydgraph.Operation(schema=schema, run_in_background=True)
    client.alter(op)


def format_schemas(schema_defs: List["BaseSchema"]) -> str:
    schemas = "\n\n".join([schema.generate_schema() for schema in schema_defs])

    types = "\n\n".join([schema.generate_type() for schema in schema_defs])

    return "\n".join(
        ["  # Type Definitions", types, "\n  # Schema Definitions", schemas]
    )


def store_schema(dynamodb, schema: "Schema"):
    table = dynamodb.Table(os.environ["BUCKET_PREFIX"] + "-grapl_schema_table")
    for f_edge, (edge_t, r_edge) in schema.get_edges().items():
        if not (f_edge and r_edge):
            LOGGER.warn(f"missing {f_edge} {r_edge} for {schema.self_type()}")
            continue
        table.put_item(
            Item={
                "f_edge": f_edge,
                "r_edge": r_edge,
                "relationship": int(edge_t.rel),
            }
        )

        table.put_item(
            Item={
                "f_edge": r_edge,
                "r_edge": f_edge,
                "relationship": int(edge_t.rel.reverse()),
            }
        )


def provision_master_graph(
    master_graph_client: GraphClient, schemas: List["BaseSchema"]
) -> None:
    mg_schema_str = format_schemas(schemas)
    set_schema(master_graph_client, mg_schema_str)


def get_dynamodb_client() -> Any:
    if IS_LOCAL:
        return boto3.resource(
            "dynamodb",
            endpoint_url="http://dynamodb:8000",
            region_name="us-east-1",
            aws_access_key_id="dummy_cred_aws_access_key_id",
            aws_secret_access_key="dummy_cred_aws_secret_access_key",
        )
    else:
        return boto3.resource("dynamodb")


def git_walker(repo, directory, f):
    f(directory)
    for path in into_list(repo.get_contents(directory.path)):
        if path.path == directory.path:
            return
        inner_directories = into_list(repo.get_contents(path.path))
        for inner_directory in inner_directories:
            git_walker(repo, inner_directory, f)


def is_schema(schema_cls):
    try:
        schema_cls.self_type()
    except Exception as e:
        LOGGER.debug(f"no self_type {e}")
        return False
    return True


def get_schema_objects(meta_globals) -> "Dict[str, BaseSchema]":
    clsmembers = [(m, c) for m, c in meta_globals.items() if inspect.isclass(c)]

    return {an[0]: an[1]() for an in clsmembers if is_schema(an[1])}


def provision_schemas(master_graph_client, raw_schemas):
    # For every schema, exec the schema
    meta_globals = {}
    for raw_schema in raw_schemas:
        exec(raw_schema, meta_globals)

    # Now fetch the schemas back from memory
    schemas = list(get_schema_objects(meta_globals).values())
    LOGGER.info(f"deploying schemas: {[s.self_type() for s in schemas]}")

    LOGGER.info("init_reverse")
    for schema in schemas:
        schema.init_reverse()

    LOGGER.info("Merge the schemas with what exists in the graph")
    dynamodb = get_dynamodb_client()
    for schema in schemas:
        store_schema(dynamodb, schema)

    LOGGER.info("Reprovision the graph")
    provision_master_graph(master_graph_client, schemas)

    for schema in schemas:
        extend_schema(dynamodb, master_graph_client, schema)

    for schema in schemas:
        store_schema(dynamodb, schema)


def query_dgraph_predicate(client: "GraphClient", predicate_name: str):
    query = f"""
        schema(pred: {predicate_name}) {{  }}
    """
    txn = client.txn(read_only=True)
    try:
        res = json.loads(txn.query(query).json)["schema"][0]
    finally:
        txn.discard()

    return res


def meta_into_edge(dynamodb, schema: "Schema", f_edge):
    table = dynamodb.Table(os.environ["BUCKET_PREFIX"] + "-grapl_schema_table")
    edge_res = table.get_item(Key={"f_edge": f_edge})["Item"]
    edge_t = schema.edges[f_edge][0]  # type: EdgeT

    return EdgeT(type(schema), edge_t.dest, EdgeRelationship(edge_res["relationship"]))


def meta_into_property(predicate_meta):
    is_set = predicate_meta.get("list")
    type_name = predicate_meta["type"]
    primitive = None
    if type_name == "string":
        primitive = PropPrimitive.Str
    if type_name == "int":
        primitive = PropPrimitive.Int
    if type_name == "bool":
        primitive = PropPrimitive.Bool

    return PropType(primitive, is_set, index=predicate_meta.get("index", []))


def meta_into_predicate(dynamodb, schema, predicate_meta):
    try:
        if predicate_meta["type"] == "uid":
            return meta_into_edge(dynamodb, schema, predicate_meta["predicate"])
        else:
            return meta_into_property(predicate_meta)
    except Exception as e:
        LOGGER.error(f"Failed to convert meta to predicate: {predicate_meta} {e}")
        raise e


def query_dgraph_type(client: "GraphClient", type_name: str):
    query = f"""
        schema(type: {type_name}) {{ type }}
    """
    txn = client.txn(read_only=True)
    try:
        res = json.loads(txn.query(query).json)
    finally:
        txn.discard()

    if not res:
        return []
    if not res.get("types"):
        return []

    res = res["types"][0]["fields"]
    predicate_names = []
    for pred in res:
        predicate_names.append(pred["name"])

    predicate_metas = []
    for predicate_name in predicate_names:
        predicate_metas.append(query_dgraph_predicate(client, predicate_name))

    return predicate_metas


def get_reverse_edge(dynamodb, schema, f_edge):
    table = dynamodb.Table(os.environ["BUCKET_PREFIX"] + "-grapl_schema_table")
    edge_res = table.get_item(Key={"f_edge": f_edge})["Item"]
    return edge_res["r_edge"]


def extend_schema(dynamodb, graph_client: GraphClient, schema: "BaseSchema"):
    predicate_metas = query_dgraph_type(graph_client, schema.self_type())
    for predicate_meta in predicate_metas:
        predicate = meta_into_predicate(dynamodb, schema, predicate_meta)
        if isinstance(predicate, PropType):
            schema.add_property(predicate_meta["predicate"], predicate)
        else:
            r_edge = get_reverse_edge(dynamodb, schema, predicate_meta["predicate"])
            schema.add_edge(predicate_meta["predicate"], predicate, r_edge)


def upload_plugin(s3_client: BaseClient, key: str, contents: str) -> Optional[Response]:
    plugin_bucket = (os.environ["BUCKET_PREFIX"] + "-model-plugins-bucket").lower()

    plugin_parts = key.split("/")
    plugin_name = plugin_parts[0]
    plugin_key = "/".join(plugin_parts[1:])

    if not (plugin_name and plugin_key):
        # if we upload a dir that looks like
        # model_plugins/
        #   __init__.py
        #   grapl_aws_model_plugin/
        #     ...lots of files...
        # we want to skip uploading the initial __init__.py, since we can't figure out what
        # plugin_name it would belong to.
        LOGGER.info(f"Skipping uploading key {key}")
        return None

    try:
        s3_client.put_object(
            Body=contents,
            Bucket=plugin_bucket,
            Key=plugin_name
            + "/"
            + base64.encodebytes((plugin_key.encode("utf8"))).decode(),
        )
    except Exception:
        msg = f"Failed to put_object to s3 {key}"
        LOGGER.error(f"{msg} {traceback.format_exc()}")
        return respond(msg)
    return None


BUCKET_PREFIX = os.environ["BUCKET_PREFIX"]


def respond(
    err, res=None, headers=None, status_code: Optional[HTTPStatus] = None
) -> Response:
    if not headers:
        headers = {}

    if IS_LOCAL:
        override = app.current_request.headers.get("origin", "")
        headers = {"Access-Control-Allow-Origin": override, **headers}
    status_code = status_code or (HTTPStatus.BAD_REQUEST if err else HTTPStatus.OK)

    return Response(
        body={"error": err} if err else json.dumps({"success": res}),
        status_code=status_code.value,
        headers={
            "Content-Type": "application/json",
            "Access-Control-Allow-Credentials": "true",
            "Access-Control-Allow-Methods": "GET,POST,OPTIONS",
            "X-Requested-With": "*",
            "Access-Control-Allow-Headers": "Content-Type, Access-Control-Allow-Headers, Authorization, X-Requested-With",
            **headers,
        },
    )


def requires_auth(path):
    if not IS_LOCAL:
        path = "/{proxy+}" + path

    def route_wrapper(route_fn):
        @app.route(path, methods=["OPTIONS", "POST"])
        def inner_route():
            if app.current_request.method == "OPTIONS":
                return respond(None, {})

            if not check_jwt(app.current_request.headers):
                LOGGER.warning("not logged in")
                return respond("Must log in", status_code=HTTPStatus.UNAUTHORIZED)
            try:
                return route_fn()
            except Exception as e:
                LOGGER.error(traceback.format_exc())
                return respond("Unexpected Error")

        return inner_route

    return route_wrapper


def no_auth(path):
    if not IS_LOCAL:
        path = "/{proxy+}" + path

    def route_wrapper(route_fn):
        @app.route(path, methods=["OPTIONS", "POST"])
        def inner_route():
            if app.current_request.method == "OPTIONS":
                return respond(None, {})
            try:
                return route_fn()
            except Exception:
                LOGGER.error(path + " failed " + traceback.format_exc())
                return respond("Unexpected Error")

        return inner_route

    return route_wrapper


def upload_plugins(s3_client, plugin_files: Dict[str, str]) -> Optional[Response]:
    plugin_files = {f: c for f, c in plugin_files.items() if not f.endswith(".pyc")}
    raw_schemas = [
        contents
        for path, contents in plugin_files.items()
        if path.endswith("schema.py") or path.endswith("schemas.py")
    ]

    py_files = {f: c for f, c in plugin_files.items() if f.endswith(".py")}

    for path, contents in py_files.items():
        directory = Path(os.path.join("/tmp/model_plugins/", os.path.dirname(path)))

        directory.mkdir(parents=True, exist_ok=True)
        with open(os.path.join("/tmp/model_plugins/", path), "w") as f:
            f.write(contents)

    th = threading.Thread(
        target=provision_schemas,
        args=(
            GraphClient(),
            raw_schemas,
        ),
    )
    th.start()

    try:
        for path, file in plugin_files.items():
            upload_resp = upload_plugin(s3_client, path, file)
            if upload_resp:
                return upload_resp
    finally:
        th.join()
    return None


@no_auth("/gitWebhook")
def webhook():
    shared_secret = os.environ["GITHUB_SHARED_SECRET"]
    access_token = os.environ["GITHUB_ACCESS_TOKEN"]

    signature = app.current_request.headers["X-Hub-Signature"]

    assert verify_payload(
        app.current_request.body.encode("utf8"), shared_secret.encode(), signature
    )

    repo_name = app.current_request.json_body["repository"]["full_name"]
    if body["ref"] != "refs/heads/master":
        return

    g = Github(access_token)

    repo = g.get_repo(repo_name)

    plugin_folders = repo.get_contents("model_plugins")
    # Upload every single file and folder, within 'plugins', to Grapl

    plugin_paths = []
    for plugin_folder in plugin_folders:
        git_walker(repo, plugin_folder, lambda file: plugin_paths.append(file))

    plugin_files = {}
    for path in plugin_paths:
        if not path.content:
            continue

        file_contents = b64decode(path.content).decode()
        plugin_files[path.path] = file_contents

    s3 = S3ClientFactory(boto3).from_env()
    upload_plugins_resp = upload_plugins(s3, plugin_files)
    if upload_plugins_resp:
        return upload_plugins_resp
    return respond(None, {})


# We expect a body of:
"""
"plugins": {
    "<plugin_path>": "<plugin_contents>",
}
"""


@requires_auth("/deploy")
def deploy():
    LOGGER.info("/deploy")
    request = app.current_request
    plugins = request.json_body.get("plugins", {})

    LOGGER.info(f"Deploying {request.json_body['plugins'].keys()}")
    upload_plugins_resp = upload_plugins(get_s3_client(), plugins)
    if upload_plugins_resp:
        return upload_plugins_resp
    LOGGER.info("uploaded plugins")
    return respond(None, {"Success": True})


def get_plugin_list(s3: BaseClient):
    plugin_bucket = (os.environ["BUCKET_PREFIX"] + "-model-plugins-bucket").lower()
    list_response = s3.list_objects_v2(Bucket=plugin_bucket)
    if not list_response.get("Contents"):
        return []

    plugin_names = set()
    for response in list_response["Contents"]:
        key = response["Key"]
        plugin_name = key.split("/")[0]
        plugin_names.add(plugin_name)
    return [plugin_name for plugin_name in plugin_names if plugin_name != "__init__.py"]


@requires_auth("/listModelPlugins")
def list_model_plugins():
    LOGGER.info("/listModelPlugins")
    try:
        plugin_names = get_plugin_list(get_s3_client())
    except Exception as e:
        LOGGER.error("failed with %s", traceback.format_exc())
        return respond({"Failed": "Failed"})

    LOGGER.info("plugin_names: %s", plugin_names)
    return respond(None, {"plugin_list": plugin_names})


def delete_plugin(s3_client, plugin_name):
    plugin_bucket = (os.environ["BUCKET_PREFIX"] + "-model-plugins-bucket").lower()

    list_response = s3_client.list_objects_v2(
        Bucket=plugin_bucket,
        Prefix=plugin_name,
    )

    if not list_response.get("Contents"):
        return []

    plugin_names = set()
    for response in list_response["Contents"]:
        s3_client.delete_object(Bucket=plugin_bucket, Key=response["Key"])


@requires_auth("/deleteModelPlugin")
def delete_model_plugin():
    try:
        LOGGER.info("/deleteModelPlugin")
        request = app.current_request
        plugins_to_delete = request.json_body.get("plugins_to_delete", [])

        s3_client = get_s3_client()
        for plugin_name in plugins_to_delete:
            delete_plugin(s3_client, plugin_name)
    except Exception as e:
        LOGGER.error(traceback.format_exc())
        return respond("deleteModelPlugin: Server Error")

    return respond(None, {"Success": "Deleted plugins"})


@app.route("/modelPluginDeployer/{proxy+}", methods=["OPTIONS", "POST"])
def nop_route():
    LOGGER.info("routing: " + app.current_request.context["path"])

    if app.current_request.method == "OPTIONS":
        return respond(None, {})

    try:
        path = app.current_request.context["path"]
        if path == "/prod/modelPluginDeployer/gitWebhook":
            return webhook()
        if path == "/prod/modelPluginDeployer/deploy":
            return deploy()
        if path == "/prod/modelPluginDeployer/listModelPlugins":
            return list_model_plugins()
        if path == "/prod/modelPluginDeployer/deleteModelPlugin":
            return delete_model_plugin()

        return respond("InvalidPath")
    except Exception:
        LOGGER.error(traceback.format_exc())
        return respond("Route Server Error")
