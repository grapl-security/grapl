from __future__ import annotations

import os
import time
from hashlib import pbkdf2_hmac, sha256
from hmac import compare_digest
from random import uniform
from typing import (
    TYPE_CHECKING,
    Any,
    Callable,
    Mapping,
    Optional,
    Tuple,
    cast,
)

import boto3  # type: ignore
import jwt
from chalice import Response  # type: ignore

from grapl_common.env_helpers import SecretsManagerClientFactory, DynamoDBResourceFactory
from grapl_common.grapl_logger import get_module_grapl_logger
from .grapl_http_service import RouteFn, respond

LOGGER = get_module_grapl_logger()

if TYPE_CHECKING:
    from mypy_boto3_dynamodb.service_resource import Table
    from mypy_boto3_dynamodb import DynamoDBServiceResource

Salt = bytes


class LazyJwtSecret:
    def __init__(self) -> None:
        self.secret: Optional[str] = None
        self.secret_id: str = os.environ["JWT_SECRET_ID"]

    def get(self) -> str:
        if self.secret is None:
            self.secret = self._retrieve_jwt_secret()
        return self.secret

    def _retrieve_jwt_secret(self) -> str:
        return self._retrieve_jwt_secret_with_retries(max_tries=1)

    def _retrieve_jwt_secret_with_retries(self, max_tries: int) -> str:
        # Theory: This whole code block is deprecated by the `wait-for-it grapl-provision`,
        # which guarantees that the JWT Secret is, now, in the secretsmanager. - wimax

        timeout_secs = max_tries or 30
        jwt_secret: Optional[str] = None

        for _ in range(timeout_secs):
            try:
                secretsmanager = SecretsManagerClientFactory(boto3).from_env()
                jwt_secret = secretsmanager.get_secret_value(
                    SecretId=self.secret_id,
                )["SecretString"]
                break
            except Exception as e:
                LOGGER.debug(e)
                time.sleep(1)
        if not jwt_secret:
            raise TimeoutError(
                f"Expected secretsmanager to be available within {timeout_secs} seconds"
            )
        return jwt_secret


_JWT_SECRET = LazyJwtSecret()


def check_jwt(headers: Mapping[str, Any]) -> bool:
    "Given headers, returns if the jwt token matches"
    encoded_jwt = None
    for cookie in headers.get("Cookie", "").split(";"):
        if "grapl_jwt=" in cookie:
            encoded_jwt = cookie.split("grapl_jwt=")[1].strip()

    if not encoded_jwt:
        LOGGER.info("encoded_jwt %s", encoded_jwt)
        return False

    try:
        jwt.decode(encoded_jwt, _JWT_SECRET.get(), algorithms=["HS256"])
        return True
    except Exception as e:
        LOGGER.error("jwt.decode %s", e)
        return False


def requires_auth(app: Any, path: str) -> Callable[[RouteFn], RouteFn]:
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


def no_auth(app: Any, path: str) -> Callable[[RouteFn], RouteFn]:
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


def login(username: str, password: str) -> Optional[str]:
    # Connect to dynamodb table
    table = _user_auth_table()

    # Get salt for username
    salt, true_pw = _get_salt_and_pw(table, username)
    if not salt or not true_pw:
        return None

    # Hash password
    to_check = _hash_password(password.encode("utf8"), salt)

    if not compare_digest(to_check, true_pw):
        time.sleep(round(uniform(0.1, 3.0), 2))
        return None

    # Use JWT to generate token
    return jwt.encode({"username": username}, _JWT_SECRET.get(), algorithm="HS256")


def login_cookie(username: str, password: str) -> Optional[str]:
    login_res = login(username, password)
    if not login_res:
        return None
    return f"grapl_jwt={login_res}; HttpOnly; path=/"


def _hash_password(cleartext: bytes, salt: Salt) -> str:
    hashed = sha256(cleartext).digest()
    return pbkdf2_hmac("sha256", hashed, salt, 512000).hex()


DYNAMO: Optional[DynamoDBServiceResource] = None


# Probably a 'LazyDynamo' is better if we want to avoid re-creating?
def _user_auth_table() -> Table:
    global DYNAMO
    DYNAMO = DYNAMO or DynamoDBResourceFactory(boto3).from_env()

    return DYNAMO.Table(os.environ["USER_AUTH_TABLE"])


def _get_salt_and_pw(
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
