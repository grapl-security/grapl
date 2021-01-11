from __future__ import annotations

import json
import logging
import os
from typing import (
    TYPE_CHECKING,
    Any,
    Callable,
    Dict,
    Optional,
    TypeVar,
    Union,
    cast,
)

import boto3
import sys
import time
from chalice import Chalice, Response

try:
    from src.lib.env_vars import UX_BUCKET_NAME, GRAPL_LOG_LEVEL, IS_LOCAL
except:
    from lib.env_vars import UX_BUCKET_NAME, GRAPL_LOG_LEVEL, IS_LOCAL

if TYPE_CHECKING:
    from mypy_boto3_s3.service_resource import (
        Bucket,
    )
    pass


LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(GRAPL_LOG_LEVEL)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))


class LazyUxBucket:
    def __init__(self) -> None:
        self.ux_bucket: Optional[Bucket] = None

    def get(self) -> Bucket:
        if self.ux_bucket is None:
            self.ux_bucket = self._retrieve_bucket()
        return self.ux_bucket

    def get_resource(self, resource_name: str) -> Optional[str]:
        bucket = self.get()
        try:
            object = bucket.Object(resource_name)
        except Exception as e:
            # TODO: We should only return None in cases where the object doesn't exist
            LOGGER.debug("Failed to retrieve object: {}", e)
            return None

        return str(object.get()['Body'].read(), 'utf8')


    def _retrieve_bucket(self) -> Bucket:
        if IS_LOCAL:
            return self._retrieve_bucket_local()
        else:
            s3 = boto3.resource('s3')
            return s3.Bucket('name')

    def _retrieve_bucket_local(self) -> Bucket:
        # Theory: This whole code block is deprecated by the `wait-for-it grapl-provision`,
        # which guarantees that the JWT Secret is, now, in the secretsmanager. - wimax

        timeout_secs = 30
        bucket: Optional[Bucket] = None

        for _ in range(timeout_secs):
            try:
                s3 = boto3.resource(
                    "s3",
                    endpoint_url="http://s3:9000",
                    aws_access_key_id="minioadmin",
                    aws_secret_access_key="minioadmin",
                )

                bucket = s3.Bucket(UX_BUCKET_NAME)
                break
            except Exception as e:
                LOGGER.debug(e)
                time.sleep(1)
        if not bucket:
            raise TimeoutError(
                f"Expected s3 ux bucket to be available within {timeout_secs} seconds"
            )
        return bucket


UX_BUCKET = LazyUxBucket()


app = Chalice(app_name="grapl-ux-edge")

if IS_LOCAL:
    app.debug = True

# Sometimes we pass in a dict. Sometimes we pass the string "True". Weird.
Res = Union[Dict[str, Any], str]


def respond(
    err: Optional[str],
    res: Optional[Res] = None,
    headers: Optional[Dict[str, Any]] = None,
) -> Response:

    LOGGER.info(f"responding, origin: {app.current_request.headers.get('origin', '')}")
    if not headers:
        headers = {}

    # TODO: We should set Cache-Control headers here
    return Response(
        body={"error": err} if err else json.dumps({"success": res}),
        status_code=400 if err else 200,
        headers={
            "Access-Control-Allow-Origin": "*",
            "Access-Control-Allow-Credentials": "false",
            "Content-Type": "application/json",
            "Access-Control-Allow-Methods": "GET,OPTIONS",
            "X-Requested-With": "*",
            "Access-Control-Allow-Headers": "Content-Type, Access-Control-Allow-Headers, Authorization, X-Requested-With",
            **headers,
        },
    )


RouteFn = TypeVar("RouteFn", bound=Callable[..., Response])


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
                LOGGER.error(f"path {path} had an error: {e}")
                return respond("Unexpected Error")

        return cast(RouteFn, inner_route)

    return route_wrapper


def fetch_from_ux_bucket():


@no_auth("/{resource}")
def route_to_resource(resource_name) -> Response:
    LOGGER.debug(f"/route_to_resource/{resource_name}")
    resource = UX_BUCKET.get_resource(resource_name)
    return Response(
        body=resource,
        status_code=200,
        headers={
            "Access-Control-Allow-Origin": "*",
            "Access-Control-Allow-Credentials": "false",
            "Content-Type": "application/json",
            "Access-Control-Allow-Methods": "GET,OPTIONS",
            "X-Requested-With": "*",
            "Access-Control-Allow-Headers": "Content-Type, Access-Control-Allow-Headers, Authorization, X-Requested-With",
        },
    )

@app.route("/{proxy+}", methods=["OPTIONS", "POST", "GET"])
def nop_route() -> Response:
    LOGGER.debug(app.current_request.context["path"])
    if app.current_request.method == "OPTIONS":
        return respond(None, {})

    path = app.current_request.context["path"]
    path_to_handler = {
        "/prod/": route_to_resource,
    }
    handler = path_to_handler.get(path, None)
    if handler:
        return handler()

    return respond(err=f"Invalid path: {path}")
