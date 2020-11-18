from __future__ import annotations

import json
import logging
import os
import re
import sys
import time
import uuid
from hashlib import pbkdf2_hmac, sha256
from hmac import compare_digest
from random import uniform
from typing import (
    TYPE_CHECKING,
    Any,
    Callable,
    Dict,
    List,
    Optional,
    Tuple,
    TypeVar,
    Union,
    cast,
)

import boto3
import jwt
import pydgraph  # type: ignore
from chalice import Chalice, CORSConfig, Response
from pydgraph import DgraphClient

from grapl_analyzerlib.nodes.base import BaseQuery, BaseView
from grapl_analyzerlib.nodes.entity import EntityQuery
from grapl_analyzerlib.nodes.lens import LensQuery

if TYPE_CHECKING:
    from mypy_boto3_dynamodb.service_resource import DynamoDBServiceResource, Table
    Salt = bytes

IS_LOCAL = bool(os.environ.get("IS_LOCAL", False))


GRAPL_LOG_LEVEL = os.getenv("GRAPL_LOG_LEVEL")
LEVEL = "ERROR" if GRAPL_LOG_LEVEL is None else GRAPL_LOG_LEVEL
LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(LEVEL)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

JWT_SECRET: Optional[str] = None

if IS_LOCAL:
    # Theory: This whole code block is deprecated by the `wait-for-it grapl-provision`, 
    # which guarantees that the JWT Secret is, now, in the secretsmanager. - wimax

    import time
    TIMEOUT_SECS = 30

    for _ in range(TIMEOUT_SECS):
        try:
            secretsmanager = boto3.client(
                "secretsmanager",
                region_name="us-east-1",
                aws_access_key_id="dummy_cred_aws_access_key_id",
                aws_secret_access_key="dummy_cred_aws_secret_access_key",
                endpoint_url="http://secretsmanager.us-east-1.amazonaws.com:4566",
            )

            JWT_SECRET = secretsmanager.get_secret_value(
                SecretId="JWT_SECRET_ID",
            )["SecretString"]
            break
        except Exception as e:
            LOGGER.debug(e)
            time.sleep(1)
    if not JWT_SECRET:
        raise TimeoutError("Expected secretsmanager to be available within {TIMEOUT_SECS} seconds")
else:
    JWT_SECRET_ID = os.environ["JWT_SECRET_ID"]

    secretsmanager = boto3.client("secretsmanager")

    JWT_SECRET = secretsmanager.get_secret_value(
        SecretId=JWT_SECRET_ID,
    )["SecretString"]

ORIGIN = os.environ["UX_BUCKET_URL"].lower()

ORIGIN_OVERRIDE = os.environ.get("ORIGIN_OVERRIDE", None)
DYNAMO: Optional[DynamoDBServiceResource] = None

if IS_LOCAL:
    MG_ALPHA = "master_graph:9080"
else:
    MG_ALPHA = "alpha0.mastergraphcluster.grapl:9080"

app = Chalice(app_name="engagement-edge")

if IS_LOCAL:
    # Locally we may want to connect from many origins
    origin_re = re.compile(
        f"http://.+/",
        re.IGNORECASE,
    )
else:
    origin_re = re.compile(
        f'https://{os.environ["BUCKET_PREFIX"]}-engagement-ux-bucket.s3[.\w\-]{1,14}amazonaws.com/',
        re.IGNORECASE,
    )


# Sometimes we pass in a dict. Sometimes we pass the string "True". Weird.
Res = Union[Dict[str, Any], str]


def respond(
    err: Optional[str],
    res: Optional[Res] = None,
    headers: Optional[Dict[str, Any]] = None,
) -> Response:
    req_origin = app.current_request.headers.get("origin", "")

    LOGGER.info(f"responding, origin: {app.current_request.headers.get('origin', '')}")
    if not headers:
        headers = {}

    if IS_LOCAL:
        override = app.current_request.headers.get("origin", "")
        LOGGER.info(f"overriding origin with {override}")
    else:
        override = ORIGIN_OVERRIDE

    if origin_re.match(req_origin):
        LOGGER.info("Origin matched")
        allow_origin = req_origin
    else:
        LOGGER.info("Origin did not match")
        # allow_origin = override or ORIGIN
        # todo: Fixme
        allow_origin = req_origin

    return Response(
        body={"error": err} if err else json.dumps({"success": res}),
        status_code=400 if err else 200,
        headers={
            "Access-Control-Allow-Origin": allow_origin,
            "Access-Control-Allow-Credentials": "true",
            "Content-Type": "application/json",
            "Access-Control-Allow-Methods": "GET,POST,OPTIONS",
            "X-Requested-With": "*",
            "Access-Control-Allow-Headers": "Content-Type, Access-Control-Allow-Headers, Authorization, X-Requested-With",
            **headers,
        },
    )


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
    if IS_LOCAL:
        DYNAMO = DYNAMO or boto3.resource(
            "dynamodb",
            region_name="us-west-2",
            endpoint_url="http://dynamodb:8000",
            aws_access_key_id="dummy_cred_aws_access_key_id",
            aws_secret_access_key="dummy_cred_aws_secret_access_key",
        )
    else:
        DYNAMO = DYNAMO or boto3.resource("dynamodb")

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
    return jwt.encode({"username": username}, JWT_SECRET, algorithm="HS256").decode(
        "utf8"
    )


def check_jwt(headers: Dict[str, Any]) -> bool:
    encoded_jwt = None
    for cookie in headers.get("Cookie", "").split(";"):
        if "grapl_jwt=" in cookie:
            encoded_jwt = cookie.split("grapl_jwt=")[1].strip()

    if not encoded_jwt:
        LOGGER.info("encoded_jwt %s", encoded_jwt)
        return False

    try:
        jwt.decode(encoded_jwt, JWT_SECRET, algorithms=["HS256"])
        return True
    except Exception as e:
        LOGGER.error("jwt.decode %s", e)
        return False


def lambda_login(event: Any) -> Optional[str]:
    body = event.json_body
    login_res = login(body["username"], body["password"])
    # Clear out the password from the dict, to avoid accidentally logging it
    body["password"] = ""
    if IS_LOCAL:
        cookie = f"grapl_jwt={login_res}; HttpOnly"
    else:
        cookie = f"grapl_jwt={login_res}; Domain=.amazonaws.com; secure; HttpOnly; SameSite=None"

    if login_res:
        return cookie
    return None


cors_config = CORSConfig(
    allow_origin=ORIGIN_OVERRIDE or ORIGIN,
    allow_credentials="true",
)

RouteFn = TypeVar("RouteFn", bound=Callable[..., Response])


def requires_auth(path: str) -> Callable[[RouteFn], RouteFn]:
    if not IS_LOCAL:
        path = "/{proxy+}" + path

    def route_wrapper(route_fn: RouteFn) -> RouteFn:
        @app.route(path, methods=["OPTIONS", "POST"])
        def inner_route() -> Response:
            if app.current_request.method == "OPTIONS":
                return respond(None, {})

            if not check_jwt(app.current_request.headers):
                LOGGER.warn("not logged in")
                return respond("Must log in")
            try:
                return route_fn()
            except Exception as e:
                LOGGER.error(e)
                return respond("Unexpected Error")

        return cast(RouteFn, inner_route)

    return route_wrapper


def no_auth(path: str) -> Callable[[RouteFn], RouteFn]:
    if not IS_LOCAL:
        path = "/{proxy+}" + path

    def route_wrapper(route_fn: RouteFn) -> RouteFn:
        @app.route(path, methods=["OPTIONS", "GET", "POST"])
        def inner_route() -> Response:
            if app.current_request.method == "OPTIONS":
                return respond(None, {})
            try:
                return route_fn()
            except Exception as e:
                LOGGER.error("path %s", e)
                return respond("Unexpected Error")

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
        LOGGER.warn("not logged in")
        return respond("Failed to login")


@no_auth("/checkLogin")
def check_login() -> Response:
    LOGGER.debug("/checkLogin %s", app.current_request)
    request = app.current_request
    if check_jwt(request.headers):
        return respond(None, "True")
    else:
        return respond(None, "False")


@app.route("/{proxy+}", methods=["OPTIONS", "POST", "GET"])
def nop_route() -> Response:
    LOGGER.debug(app.current_request.context["path"])

    path = app.current_request.context["path"]

    if path == "/prod/login":
        return login_route()
    elif path == "/prod/checkLogin":
        return check_login()

    return respond("InvalidPath")
