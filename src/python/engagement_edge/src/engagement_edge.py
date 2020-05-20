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
    os.environ["JWT_SECRET"] = str(uuid.uuid4())
    os.environ["BUCKET_PREFIX"] = "local-grapl"

JWT_SECRET = os.environ["JWT_SECRET"]
ORIGIN = (
    "https://" + os.environ["BUCKET_PREFIX"] + "engagement-ux-bucket.s3.amazonaws.com"
)
ORIGIN_OVERRIDE = os.environ.get("ORIGIN_OVERRIDE", None)
# ORIGIN = "http://127.0.0.1:8900"
DYNAMO = None

GRAPL_LOG_LEVEL = os.getenv("GRAPL_LOG_LEVEL")
LEVEL = "ERROR" if GRAPL_LOG_LEVEL is None else GRAPL_LOG_LEVEL
logging.basicConfig(stream=sys.stdout, level=LEVEL)
LOGGER = logging.getLogger("engagement-creator")

app = Chalice(app_name="engagement-edge")

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
    if IS_LOCAL:
        return True

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
    cookie = f"grapl_jwt={login_res}; secure; HttpOnly; SameSite=None"
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

            if not IS_LOCAL:  # For now, disable authentication locally
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
