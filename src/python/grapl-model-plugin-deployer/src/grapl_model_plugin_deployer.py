from __future__ import annotations

import base64
import hmac
import inspect
import json
import os
import sys
import threading
import traceback
from base64 import b64decode
from hashlib import sha1
from http import HTTPStatus
from pathlib import Path
from typing import (
    TYPE_CHECKING,
    Any,
    Callable,
    Dict,
    List,
    Mapping,
    Optional,
    Type,
    TypeVar,
    Union,
    cast,
)

import boto3  # type: ignore
import jwt
import pydgraph  # type: ignore
from chalice import Chalice, Response
from github import Github
from grapl_analyzerlib.node_types import (
    EdgeRelationship,
    EdgeT,
    PropPrimitive,
    PropType,
)
from grapl_analyzerlib.prelude import *
from grapl_analyzerlib.schema import Schema
from grapl_common.env_helpers import (
    DynamoDBResourceFactory,
    S3ClientFactory,
    SecretsManagerClientFactory,
)
from grapl_common.grapl_logger import get_module_grapl_logger
from grapl_common.provision import (
    store_schema_properties as store_schema_properties_common,
)

sys.path.append("/tmp/")

T = TypeVar("T")

IS_LOCAL = bool(os.environ.get("IS_LOCAL", False))

LOGGER = get_module_grapl_logger(default_log_level="ERROR")

if TYPE_CHECKING:
    from mypy_boto3_dynamodb import DynamoDBServiceResource
    from mypy_boto3_s3 import S3Client

try:
    directory = Path("/tmp/model_plugins/")
    directory.mkdir(parents=True, exist_ok=True)
except Exception as e:
    LOGGER.error("Failed to create directory", e)

if IS_LOCAL:
    import time

    for i in range(0, 150):
        try:
            secretsmanager = SecretsManagerClientFactory(boto3).from_env()
            JWT_SECRET = secretsmanager.get_secret_value(
                SecretId="JWT_SECRET_ID",
            )["SecretString"]
            break
        except Exception as e:
            LOGGER.debug(e)
            time.sleep(1)

    os.environ["DEPLOYMENT_NAME"] = "local-grapl"
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


def check_jwt(headers: Mapping[str, Any]) -> bool:
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


def store_schema(dynamodb, schema: "Schema") -> None:
    grapl_schema_table = dynamodb.Table(
        os.environ["DEPLOYMENT_NAME"] + "-grapl_schema_table"
    )
    grapl_schema_properties = dynamodb.Table(
        os.environ["DEPLOYMENT_NAME"] + "-grapl_schema_properties"
    )

    grapl_schema_properties.put_item(
        Item={
            "node_type": schema.self_type(),
            "display_property": schema.get_display_property(),
        }
    )
    for f_edge, (edge_t, r_edge) in schema.get_edges().items():
        if not (f_edge and r_edge):
            LOGGER.warn(f"missing {f_edge} {r_edge} for {schema.self_type()}")
            continue
        grapl_schema_table.put_item(
            Item={
                "f_edge": f_edge,
                "r_edge": r_edge,
                "relationship": int(edge_t.rel),
            }
        )

        grapl_schema_table.put_item(
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


def is_schema(schema_cls: Type[Schema]) -> bool:
    try:
        schema_cls.self_type()
    except Exception as e:
        LOGGER.debug(f"no self_type {e}")
        return False
    return True


def get_schema_objects(meta_globals) -> "Dict[str, BaseSchema]":
    clsmembers = [(m, c) for m, c in meta_globals.items() if inspect.isclass(c)]

    return {an[0]: an[1]() for an in clsmembers if is_schema(an[1])}


def store_schema_properties(dynamodb: DynamoDBServiceResource, schema: Schema) -> None:
    table = dynamodb.Table(
        os.environ["DEPLOYMENT_NAME"] + "-grapl_schema_properties_table"
    )
    store_schema_properties_common(table, schema)


def provision_schemas(master_graph_client, raw_schemas):
    # For every schema, exec the schema
    meta_globals: Dict = {}
    for raw_schema in raw_schemas:
        exec(raw_schema, meta_globals)

    # Now fetch the schemas back from memory
    schemas = list(get_schema_objects(meta_globals).values())
    LOGGER.info(f"deploying schemas: {[s.self_type() for s in schemas]}")

    LOGGER.info("init_reverse")
    for schema in schemas:
        schema.init_reverse()

    LOGGER.info("Merge the schemas with what exists in the graph")
    dynamodb = DynamoDBResourceFactory(boto3).from_env()
    for schema in schemas:
        store_schema(dynamodb, schema)

    LOGGER.info("Reprovision the graph")
    provision_master_graph(master_graph_client, schemas)

    for schema in schemas:
        extend_schema(dynamodb, master_graph_client, schema)

    for schema in schemas:
        store_schema(dynamodb, schema)
        store_schema_properties(dynamodb, schema)


def query_dgraph_predicate(client: "GraphClient", predicate_name: str) -> Any:
    query = f"""
        schema(pred: {predicate_name}) {{  }}
    """
    txn = client.txn(read_only=True)
    try:
        res = json.loads(txn.query(query).json)["schema"][0]
    finally:
        txn.discard()

    return res


def meta_into_edge(dynamodb, schema: "Schema", f_edge) -> EdgeT:
    table = dynamodb.Table(os.environ["DEPLOYMENT_NAME"] + "-grapl_schema_table")
    edge_res = table.get_item(Key={"f_edge": f_edge})["Item"]
    edge_t = schema.edges[f_edge][0]  # type: EdgeT

    return EdgeT(type(schema), edge_t.dest, EdgeRelationship(edge_res["relationship"]))


def meta_into_property(predicate_meta) -> PropType:
    is_set = predicate_meta.get("list")
    type_name = predicate_meta["type"]
    primitive = None
    if type_name == "string":
        primitive = PropPrimitive.Str
    if type_name == "int":
        primitive = PropPrimitive.Int
    if type_name == "bool":
        primitive = PropPrimitive.Bool

    assert primitive is not None
    return PropType(primitive, is_set, index=predicate_meta.get("index", []))


def meta_into_predicate(dynamodb, schema, predicate_meta) -> Union[EdgeT, PropType]:
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
    table = dynamodb.Table(os.environ["DEPLOYMENT_NAME"] + "-grapl_schema_table")
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


def upload_plugin(s3_client: S3Client, key: str, contents: str) -> Optional[Response]:
    plugin_bucket = (os.environ["DEPLOYMENT_NAME"] + "-model-plugins-bucket").lower()

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
            Body=contents.encode("utf-8"),
            Bucket=plugin_bucket,
            Key=plugin_name
            + "/"
            + base64.encodebytes((plugin_key.encode("utf8"))).decode(),
        )
        return respond(err=None, res={}, status_code=200)
    except Exception as e:
        msg = f"Failed to put_object to s3 {key}"
        LOGGER.error(msg, e)
        return respond(
            err=msg,
        )


DEPLOYMENT_NAME = os.environ["DEPLOYMENT_NAME"]


# Sometimes we pass in a dict. Sometimes we pass the string "True". Weird.
Res = Union[Dict[str, Any], str]


def respond(
    err: Optional[str],
    res: Optional[Res] = None,
    headers: Optional[Dict[str, Any]] = None,
    status_code: int = 500,
) -> Response:
    """
    This function is copy-pasted-shared between
    - engagement_edge.py
    - grapl_model_plugin_deployer.py

    Please update the other one if you update this function.

    # Q&A
    "Why not refactor it into grapl-common or someplace?"
    We are removing Chalice soon; that seems like the right time to do that change.
    """

    if not headers:
        headers = {}

    if IS_LOCAL:  # Overwrite headers
        override = app.current_request.headers.get("origin", "")
        LOGGER.warning(f"overriding origin for IS_LOCAL:\t'[{override}]")
        headers = {"Access-Control-Allow-Origin": override, **headers}

    if not err:  # Set response format for success
        body = json.dumps({"success": res})
        status_code = 200
    else:
        body = json.dumps({"error": err}) if err else json.dumps({"success": res})

    headers = {
        "Access-Control-Allow-Credentials": "true",
        "Content-Type": "application/json",
        "Access-Control-Allow-Methods": "GET,POST,OPTIONS",
        "X-Requested-With": "*",
        "Access-Control-Allow-Headers": ":authority, Content-Type, Access-Control-Allow-Headers, Authorization, X-Requested-With",
        **headers,
    }

    response = Response(
        body=body,
        status_code=status_code,
        headers=headers,
    )

    return response


RouteFn = TypeVar("RouteFn", bound=Callable[..., Response])


def requires_auth(path: str) -> Callable[[RouteFn], RouteFn]:
    if not IS_LOCAL:
        path = "/{proxy+}" + path

    def route_wrapper(route_fn: RouteFn) -> RouteFn:
        @app.route(path, methods=["OPTIONS", "POST"])
        def inner_route():
            if app.current_request.method == "OPTIONS":
                return respond(err=None, res={})

            if not check_jwt(app.current_request.headers):
                LOGGER.warning("not logged in")
                return respond(err="Must log in", status_code=HTTPStatus.UNAUTHORIZED)
            try:
                return route_fn()
            except Exception as e:
                LOGGER.error(f"Route {path} failed", e)
                return respond(err="Unexpected error, see Model Plugin Deployer logs")

        return cast(RouteFn, inner_route)

    return route_wrapper


def no_auth(path: str) -> Callable[[RouteFn], RouteFn]:
    if not IS_LOCAL:
        path = "/{proxy+}" + path

    def route_wrapper(route_fn: RouteFn) -> RouteFn:
        @app.route(path, methods=["OPTIONS", "POST"])
        def inner_route() -> Response:
            if app.current_request.method == "OPTIONS":
                return respond(err=None, res={})
            try:
                return route_fn()
            except Exception as e:
                LOGGER.error(f"Route {path} failed ", e)
                return respond(err="Unexpected error, see Model Plugin Deployer logs")

        return cast(RouteFn, inner_route)

    return route_wrapper


def upload_plugins(
    s3_client: S3Client, plugin_files: Dict[str, str]
) -> Optional[Response]:
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


# We expect a body of:
"""
"plugins": {
    "<plugin_path>": "<plugin_contents>",
}
"""


@requires_auth("/deploy")
def deploy() -> Response:
    LOGGER.info("/deploy")
    s3 = S3ClientFactory(boto3).from_env()
    request = app.current_request
    plugins = request.json_body.get("plugins", {})

    LOGGER.info(f"Deploying {request.json_body['plugins'].keys()}")
    upload_plugins_resp = upload_plugins(s3, plugins)
    if upload_plugins_resp:
        return upload_plugins_resp
    LOGGER.info("uploaded plugins")
    return respond(err=None, res={"Success": True})


def get_plugin_list(s3: S3Client) -> List[str]:
    plugin_bucket = (os.environ["DEPLOYMENT_NAME"] + "-model-plugins-bucket").lower()
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
@requires_auth("/{proxy+}/listModelPlugins")
def list_model_plugins() -> Response:
    LOGGER.info("/listModelPlugins")
    s3 = S3ClientFactory(boto3).from_env()
    try:
        plugin_names = get_plugin_list(s3)
    except Exception as e:
        msg = "list_model_plugins failed, see logs"
        LOGGER.error(msg, e)
        return respond(err=msg)

    LOGGER.info("plugin_names: %s", plugin_names)
    return respond(err=None, res={"plugin_list": plugin_names})


def delete_plugin(s3_client, plugin_name) -> None:
    plugin_bucket = (os.environ["DEPLOYMENT_NAME"] + "-model-plugins-bucket").lower()

    list_response = s3_client.list_objects_v2(
        Bucket=plugin_bucket,
        Prefix=plugin_name,
    )

    if not list_response.get("Contents"):
        return

    for response in list_response["Contents"]:
        s3_client.delete_object(Bucket=plugin_bucket, Key=response["Key"])


@requires_auth("/deleteModelPlugin")
def delete_model_plugin() -> Response:
    s3_client = S3ClientFactory(boto3).from_env()
    try:
        LOGGER.info("/deleteModelPlugin")
        request = app.current_request
        plugins_to_delete = request.json_body.get("plugins_to_delete", [])

        for plugin_name in plugins_to_delete:
            delete_plugin(s3_client, plugin_name)
    except Exception as e:
        msg = "delete_model_plugin failed, see logs"
        LOGGER.error(msg, e)
        return respond(err=msg)

    return respond(err=None, res={"Success": "Deleted plugins"})


@app.route("/prod/modelPluginDeployer/{proxy+}", methods=["OPTIONS", "PUT", "POST"])
def prod_nop_route() -> Response:
    LOGGER.info("prod_nop_route: " + app.current_request.context["path"])

    if app.current_request.method == "OPTIONS":
        return respond(None, {})

    try:
        path = app.current_request.context["path"]
        if path == "/prod/modelPluginDeployer/deploy":
            return deploy()
        if path == "/prod/modelPluginDeployer/listModelPlugins":
            return list_model_plugins()
        if path == "/prod/modelPluginDeployer/deleteModelPlugin":
            return delete_model_plugin()

        return respond(err="InvalidPath")
    except Exception:
        LOGGER.error(traceback.format_exc())
        return respond(err="Route Server Error")


@app.route("/modelPluginDeployer/{proxy+}", methods=["OPTIONS", "POST"])
def nop_route() -> Response:
    LOGGER.info("nop_route: " + app.current_request.context["path"])

    if app.current_request.method == "OPTIONS":
        return respond(err=None, res={})

    path = app.current_request.context["path"]
    path_to_handler = {
        "/prod/modelPluginDeployer/deploy": deploy,
        "/prod/modelPluginDeployer/listModelPlugins": list_model_plugins,
        "/prod/modelPluginDeployer/deleteModelPlugin": delete_model_plugin,
        "/modelPluginDeployer/deploy": deploy,
        "/modelPluginDeployer/listModelPlugins": list_model_plugins,
        "/modelPluginDeployer/deleteModelPlugin": delete_model_plugin,
    }
    handler = path_to_handler.get(path, None)
    if handler:
        return handler()

    return respond(err=f"Invalid path: {path}", status_code=HTTPStatus.NOT_FOUND)
