from __future__ import annotations

import base64
import concurrent.futures
import inspect
import json
import os
import sys
import traceback
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

import boto3
import jwt
from chalice import Chalice, Response
from grapl_analyzerlib.prelude import *
from grapl_analyzerlib.provision import provision_common
from grapl_analyzerlib.schema import Schema
from grapl_common.env_helpers import (
    DynamoDBResourceFactory,
    S3ClientFactory,
    SecretsManagerClientFactory,
    get_deployment_name,
)
from grapl_common.grapl_logger import get_module_grapl_logger
from grapl_common.utils.primitive_convertors import to_bool

sys.path.append("/tmp/")

T = TypeVar("T")

IS_LOCAL = to_bool(os.environ.get("IS_LOCAL", False))

LOGGER = get_module_grapl_logger(default_log_level="ERROR")

MODEL_PLUGINS_BUCKET = os.environ["GRAPL_MODEL_PLUGINS_BUCKET"]

if TYPE_CHECKING:
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


def get_schema_objects(meta_globals: Dict[str, Any]) -> Dict[str, BaseSchema]:
    def is_schema(schema_cls: Type[Schema]) -> bool:
        """
        A poor, but functional, heuristic to figure out if something seems like a schema.
        """
        try:
            schema_cls.self_type()
        except Exception as e:
            LOGGER.debug(f"no self_type {e}")
            return False
        return True

    clsmembers = [(m, c) for m, c in meta_globals.items() if inspect.isclass(c)]

    return {an[0]: an[1]() for an in clsmembers if is_schema(an[1])}


def provision_schemas(graph_client: GraphClient, raw_schemas: List[bytes]) -> None:
    """
    `raw_schemas` is a list of raw, exec'able python code contained in a `schema.py` file
    """
    deployment_name = get_deployment_name()

    # For every schema, exec the schema. The new schemas in scope in the file
    # are then written to `meta_globals`.
    meta_globals: Dict[str, Any] = {}
    for raw_schema in raw_schemas:
        exec(raw_schema, meta_globals)

    # Now fetch the schemas back, as Python classes, from meta_globals
    schemas = list(get_schema_objects(meta_globals).values())
    LOGGER.info(f"deploying schemas: {[s.self_type() for s in schemas]}")

    LOGGER.info("init_reverse")
    for schema in schemas:
        schema.init_reverse()

    dynamodb = DynamoDBResourceFactory(boto3).from_env()
    schema_table = provision_common.get_schema_table(
        dynamodb, deployment_name=deployment_name
    )
    schema_properties_table = provision_common.get_schema_properties_table(
        dynamodb, deployment_name=deployment_name
    )

    LOGGER.info("Merge the schemas with what exists in the graph")
    for schema in schemas:
        provision_common.store_schema(schema_table, schema)
        provision_common.store_schema_properties(schema_properties_table, schema)

    LOGGER.info("Reprovision the graph")
    schema_str = provision_common.format_schemas(schemas)
    provision_common.set_schema(graph_client, schema_str)

    for schema in schemas:
        provision_common.extend_schema(schema_table, graph_client, schema)

    for schema in schemas:
        provision_common.store_schema(schema_table, schema)
        provision_common.store_schema_properties(schema_properties_table, schema)


def upload_plugin(s3_client: S3Client, key: str, contents: str) -> Optional[Response]:
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
            Bucket=MODEL_PLUGINS_BUCKET,
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
        def inner_route() -> Response:
            if app.current_request.method == "OPTIONS":
                return respond(err=None, res={})

            if not check_jwt(app.current_request.headers):
                LOGGER.warning("not logged in")
                return respond(err="Must log in", status_code=HTTPStatus.UNAUTHORIZED)
            try:
                return route_fn()
            except Exception:
                LOGGER.error(f"Route {path} failed", exc_info=True)
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
            except Exception:
                LOGGER.error(f"Route {path} failed ", exc_info=True)
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

    # Since we need to communicate data from the `provision_schemas`
    # - namely, exceptions - concurrent.futures is more idiomatic than
    # threading.Thread
    with concurrent.futures.ThreadPoolExecutor(max_workers=1) as executor:
        provision_schema_fut = executor.submit(
            provision_schemas, GraphClient(), raw_schemas
        )

        try:
            for path, file in plugin_files.items():
                upload_resp = upload_plugin(s3_client, path, file)
                if upload_resp:
                    return upload_resp
        finally:
            for completed_future in concurrent.futures.as_completed(
                [provision_schema_fut]
            ):
                # This will also propagate any exceptions from that thread into the main thread
                completed_future.result()
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
    list_response = s3.list_objects_v2(Bucket=MODEL_PLUGINS_BUCKET)
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


def delete_plugin(s3_client: S3Client, plugin_name: str) -> None:
    list_response = s3_client.list_objects_v2(
        Bucket=MODEL_PLUGINS_BUCKET,
        Prefix=plugin_name,
    )

    if not list_response.get("Contents"):
        return

    for response in list_response["Contents"]:
        s3_client.delete_object(Bucket=MODEL_PLUGINS_BUCKET, Key=response["Key"])


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
