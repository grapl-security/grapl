from __future__ import annotations

import json
import logging
import os
import sys
import time
from hashlib import pbkdf2_hmac, sha256
from hmac import compare_digest
from random import uniform
from typing import (
    TYPE_CHECKING,
    Any,
    Callable,
    Dict,
    Mapping,
    Optional,
    Tuple,
    TypeVar,
    Union,
    cast,
)

import boto3
import jwt
from botocore.client import Config
from chalice import Chalice, Response
from engagement_edge.sagemaker import create_sagemaker_client
from grapl_common.debugger.vsc_debugger import wait_for_vsc_debugger
from grapl_common.env_helpers import (
    DynamoDBResourceFactory,
    SecretsManagerClientFactory,
)

if TYPE_CHECKING:
    from mypy_boto3_dynamodb.service_resource import DynamoDBServiceResource, Table

    Salt = bytes

LOGGER = logging.getLogger(__name__)
GRAPL_LOG_LEVEL = os.environ.get("GRAPL_LOG_LEVEL", "ERROR")
LOGGER.setLevel(GRAPL_LOG_LEVEL)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

wait_for_vsc_debugger(service="engagement_edge")


class LazyJwtSecret:
    def __init__(self) -> None:
        self.secret: Optional[str] = None

    def get(self) -> str:
        if self.secret is None:
            self.secret = self._retrieve_jwt_secret()
        return self.secret

    def _retrieve_jwt_secret(self) -> str:
        jwt_secret_id = os.environ["JWT_SECRET_ID"]

        secretsmanager = SecretsManagerClientFactory(boto3).from_env()

        jwt_secret: str = secretsmanager.get_secret_value(
            SecretId=jwt_secret_id,
        )["SecretString"]
        return jwt_secret


JWT_SECRET = LazyJwtSecret()

NOTEBOOK_NAME = os.environ["GRAPL_NOTEBOOK_INSTANCE"]

DYNAMO: Optional[DynamoDBServiceResource] = None

app = Chalice(app_name="engagement-edge")
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


def get_salt_and_pw(
    table: Table, username: str
) -> Tuple[Optional[Salt], Optional[str]]:
    LOGGER.info(f"Getting salt for user: {username}")
    response = table.get_item(
        Key={
            "username": username,
        }
    )

    if not response.get("Item"):
        LOGGER.debug(f"Did not get salt for user: {username}")
        return None, None

    # Not quite sure what type this is supposed to be.
    salt = response["Item"]["salt"].value  # type: ignore
    password = cast(str, response["Item"]["password"])
    return salt, password


def hash_password(cleartext: bytes, salt: Salt) -> str:
    hashed = sha256(cleartext).digest()
    return pbkdf2_hmac("sha256", hashed, salt, 512000).hex()


def user_auth_table() -> Table:
    global DYNAMO
    DYNAMO = DYNAMO or DynamoDBResourceFactory(boto3).from_env(
        config=Config(
            connect_timeout=5,
            read_timeout=5,
        )
    )

    return DYNAMO.Table(os.environ["USER_AUTH_TABLE"])


def login(username: str, password: str) -> Optional[str]:
    # Connect to dynamodb table
    table = user_auth_table()

    # Get salt for username
    salt, true_pw = get_salt_and_pw(table, username)
    if not salt or not true_pw:
        return None

    # Hash password
    to_check = hash_password(password.encode("utf8"), salt)

    if not compare_digest(to_check, true_pw):
        time.sleep(round(uniform(0.1, 3.0), 2))
        return None

    # Use JWT to generate token
    return jwt.encode({"username": username}, JWT_SECRET.get(), algorithm="HS256")


def check_jwt(headers: Mapping[str, Any]) -> bool:
    encoded_jwt = None
    for cookie in headers.get("Cookie", "").split(";"):
        if "grapl_jwt=" in cookie:
            encoded_jwt = cookie.split("grapl_jwt=")[1].strip()

    if not encoded_jwt:
        LOGGER.info("encoded_jwt %s", encoded_jwt)
        return False

    try:
        jwt.decode(encoded_jwt, JWT_SECRET.get(), algorithms=["HS256"])
        return True
    except Exception as e:
        LOGGER.error("jwt.decode %s", e)
        return False


def lambda_login(event: Any) -> Optional[str]:
    body = json.loads(
        event.raw_body.decode()
    )  # 'json_body' is a more natural choice, but has issues:  c.f. github issue aws/chalice#1188
    login_res = login(body["username"], body["password"])
    # Clear out the password from the dict, to avoid accidentally logging it
    body["password"] = ""
    cookie = f"grapl_jwt={login_res}; secure; HttpOnly; SameSite=None; path=/"

    if login_res:
        return cookie
    return None


RouteFn = TypeVar("RouteFn", bound=Callable[..., Response])


def requires_auth(path: str) -> Callable[[RouteFn], RouteFn]:
    def route_wrapper(route_fn: RouteFn) -> RouteFn:
        @app.route(path, methods=["OPTIONS", "POST"])
        def inner_route() -> Response:
            if app.current_request.method == "OPTIONS":
                return respond(None, {})

            if not check_jwt(app.current_request.headers):
                LOGGER.warning("not logged in")
                return respond("Must log in", status_code=403)
            try:
                return route_fn()
            except Exception as e:
                LOGGER.error(f"path {path} had an error: {e}")
                return respond(str(e))

        return cast(RouteFn, inner_route)

    return route_wrapper


def no_auth(path: str) -> Callable[[RouteFn], RouteFn]:
    def route_wrapper(route_fn: RouteFn) -> RouteFn:
        @app.route(path, methods=["OPTIONS", "GET", "POST"])
        def inner_route() -> Response:
            if app.current_request.method == "OPTIONS":
                return respond(None, {})
            try:
                return route_fn()
            except Exception as e:
                LOGGER.error(f"path {path} had an error: {e}")
                return respond(str(e))

        return cast(RouteFn, inner_route)

    return route_wrapper


@no_auth("/login")
def login_route() -> Response:
    LOGGER.debug("/login_route")
    request = app.current_request
    cookie = lambda_login(request)
    if cookie:
        LOGGER.info("logged in")
        return respond(None, "True", headers={"Set-Cookie": cookie})
    else:
        LOGGER.warning("not logged in")
        return respond("Failed to login", status_code=403)


@no_auth("/checkLogin")
def check_login() -> Response:
    LOGGER.debug(f"/checkLogin {app.current_request}")
    request = app.current_request

    if check_jwt(request.headers):
        return respond(None, "True")
    else:
        return respond(None, "False")


@requires_auth("/getNotebook")
def get_notebook() -> Response:
    client = create_sagemaker_client()
    url = client.get_presigned_url(NOTEBOOK_NAME)
    return respond(err=None, res={"notebook_url": url})


@app.route("/prod/auth/{proxy+}", methods=["OPTIONS", "POST", "GET"])
def prod_nop_route() -> Response:
    LOGGER.debug(f'prod_nop_route {app.current_request.context["path"]}')
    if app.current_request.method == "OPTIONS":
        return respond(None, {})

    LOGGER.debug(f"current_request {app.current_request.to_dict()}")
    path = app.current_request.context["path"]
    path_to_handler = {
        "/prod/auth/login": login_route,
        "/prod/auth/checkLogin": check_login,
        "/prod/auth/getNotebook": get_notebook,
        "/auth/login": login_route,
        "/auth/checkLogin": check_login,
        "/auth/getNotebook": get_notebook,
    }
    handler = path_to_handler.get(path, None)
    if handler:
        return handler()

    return respond(err=f"Invalid path: {path}", status_code=404)


@app.route("/auth/{proxy+}", methods=["OPTIONS", "POST", "GET"])
def nop_route() -> Response:
    LOGGER.debug(f'nop_route {app.current_request.context["path"]}')
    if app.current_request.method == "OPTIONS":
        return respond(None, {})

    LOGGER.debug(f"current_request {app.current_request.to_dict()}")
    path = app.current_request.context["path"]
    path_to_handler = {
        "/prod/auth/login": login_route,
        "/prod/auth/checkLogin": check_login,
        "/prod/auth/getNotebook": get_notebook,
        "/auth/login": login_route,
        "/auth/checkLogin": check_login,
        "/auth/getNotebook": get_notebook,
    }
    handler = path_to_handler.get(path, None)
    if handler:
        return handler()

    return respond(err=f"Invalid path: {path}", status_code=404)
