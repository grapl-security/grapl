from __future__ import annotations

import json
import os
import time
from typing import TYPE_CHECKING, Optional, Union

import boto3
from chalice import Response

from grapl_common.env_helpers import S3ResourceFactory
from grapl_common.grapl_logger import get_module_grapl_logger
from grapl_http_service.grapl_http_service import init, not_found, respond

if TYPE_CHECKING:
    from mypy_boto3_s3.service_resource import Bucket

IS_LOCAL = bool(os.environ.get("IS_LOCAL", False))
UX_BUCKET_NAME = os.environ["UX_BUCKET_NAME"]

LOGGER = get_module_grapl_logger()

# Must never hold more than 15 values
MEDIA_TYPE_MAP = {
    "json": "application/json",
    "ico": "image/x-icon",
    "png": "image/png",
    "html": "text/html",
    "txt": "text/plain",
    "css": "text/css",
    "js": "text/javascript",
    "chunk.js": "text/javascript",
    "chunk.css": "text/css",
    "map": "application/json",
    "": "application/octet-stream",
}

if IS_LOCAL:
    assert len(MEDIA_TYPE_MAP) < 15


class LazyUxBucket:
    def __init__(self) -> None:
        self.ux_bucket: Optional[Bucket] = None

    def get(self) -> Bucket:
        if self.ux_bucket is None:
            self.ux_bucket = self._retrieve_bucket()
        return self.ux_bucket

    def get_resource(self, resource_name: str) -> Optional[bytes]:
        bucket = self.get()
        start = int(time.time())
        try:
            obj = bucket.Object(resource_name)
            end = int(time.time())
            LOGGER.debug(f"retrieved object {resource_name} after {end - start}")
        except Exception as e:
            # TODO: We should only return None in cases where the object doesn't exist
            end = int(time.time())
            LOGGER.warning(f"Failed to retrieve object: {e} after {end - start}")
            return None

        # todo: We could just compress right here instead of allocating this intermediary
        # Or we could compress the files in s3?
        return obj.get()["Body"].read()

    def _retrieve_bucket(self) -> Bucket:
        if IS_LOCAL:
            return self._retrieve_bucket_local()
        else:
            s3 = S3ResourceFactory(boto3).from_env()
            return s3.Bucket(UX_BUCKET_NAME)

    def _retrieve_bucket_local(self) -> Bucket:
        timeout_secs = 30
        bucket: Optional[Bucket] = None

        for _ in range(timeout_secs):
            try:
                s3 = S3ResourceFactory(boto3).from_env()
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

app = init(app_name="grapl-ux-edge")

# If we ever have more than 16 binary types we need to
# instead explicitly set it for every response
# https://aws.github.io/chalice/api.html#APIGateway.binary_types
if len(MEDIA_TYPE_MAP) >= 14:
    LOGGER.error("MEDIA_TYPE_MAP length is too high")
elif len(MEDIA_TYPE_MAP) >= 13:
    LOGGER.warning("MEDIA_TYPE_MAP length is too high")
# for _media_type in MEDIA_TYPE_MAP.values():
#     app.api.binary_types.append(_media_type)


def get_media_type(resource_name: str) -> str:
    name_parts = resource_name.split(".")
    for i, _name_part in enumerate(name_parts):
        name = ".".join(name_parts[i:])
        media_type = MEDIA_TYPE_MAP.get(name)
        if media_type:
            return media_type
    return "application/octet-stream"


def _route_to_resource(resource_name: str) -> Response:
    resource = UX_BUCKET.get_resource(resource_name)
    if not resource:
        return not_found()
    content_type = get_media_type(resource_name)
    LOGGER.debug(
        f"setting content-type:  content_type: {content_type} resource_name: {resource_name}"
    )

    if content_type.startswith("text/"):
        resource = resource.decode('utf8')
    elif content_type == "application/json":
        resource = json.loads(resource.decode('utf8'))

    return Response(
        body=resource,
        status_code=200,
        headers={
            "Access-Control-Allow-Origin": "*",
            "Access-Control-Allow-Credentials": "false",
            "Content-Type": content_type,
            "Cache-Control": "max-age=60",
            "Access-Control-Allow-Methods": "GET,OPTIONS",
            "X-Requested-With": "*",
            "Access-Control-Allow-Headers": "Content-Encoding, Content-Type, Access-Control-Allow-Headers, Authorization, X-Requested-With",
        },
    )


@app.route("/prod/{proxy+}", methods=["OPTIONS", "GET"])
def prod_nop_route() -> Response:
    LOGGER.info(f'nop_route {app.current_request.context["path"]}')
    if app.current_request.method == "OPTIONS":
        return respond(None, {})

    path = app.current_request.context["path"]
    if path == "/prod/":
        return _route_to_resource("index.html")
    elif path.startswith("/prod/"):
        resource_name = path.split("/prod/")[1]
        return _route_to_resource(resource_name)
    else:
        return _route_to_resource(path)


@app.route("/{proxy+}", methods=["OPTIONS", "GET"])
def nop_route() -> Response:
    LOGGER.info(f'nop_route {app.current_request.context["path"]}')
    if app.current_request.method == "OPTIONS":
        return respond(None, {})

    path = app.current_request.context["path"]
    if path == "/prod/":
        return _route_to_resource("index.html")
    elif path.startswith("/prod/"):
        resource_name = path.split("/prod/")[1]
        return _route_to_resource(resource_name)
    else:
        return _route_to_resource(path)


@app.route("/", methods=["OPTIONS", "GET"])
def root_nop_route() -> Response:
    LOGGER.info(f'root_nop_route {app.current_request.context["path"]}')
    if app.current_request.method == "OPTIONS":
        return respond(None, {})
    return _route_to_resource("index.html")


if IS_LOCAL:

    @app.route("/static/js/{proxy+}", methods=["OPTIONS", "GET"])
    def static_js_resource_root_nop_route() -> Response:
        LOGGER.info(f'static_js_resource {app.current_request.context["path"]}')
        if app.current_request.method == "OPTIONS":
            return respond(None, {})
        return _route_to_resource(app.current_request.context["path"].lstrip("/"))


    @app.route("/static/css/{proxy+}", methods=["OPTIONS", "GET"])
    def static_css_resource_root_nop_route() -> Response:
        LOGGER.info(f'static_css_resource {app.current_request.context["path"]}')
        if app.current_request.method == "OPTIONS":
            return respond(None, {})
        return _route_to_resource(app.current_request.context["path"].lstrip("/"))


    @app.route("/static/media/{proxy+}", methods=["OPTIONS", "GET"])
    def static_media_resource_root_nop_route() -> Response:
        LOGGER.info(f'static_media_resource {app.current_request.context["path"]}')
        if app.current_request.method == "OPTIONS":
            return respond(None, {})
        return _route_to_resource(app.current_request.context["path"].lstrip("/"))
