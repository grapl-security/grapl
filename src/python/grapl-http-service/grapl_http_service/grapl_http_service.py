import os
import json

from typing import Any, Dict, Union, Optional, \
    TypeVar, Callable, cast  # todo: the aws-lambda-typing package also has type annotations for responses

from chalice import Chalice as HttpService, Response  # type: ignore

import gzip as web_compress


def init(app_name: str) -> HttpService:
    http_service: Any = HttpService(app_name=app_name)
    should_debug: Optional[str] = os.environ.get('HTTP_SERVICE_DEBUG')
    if should_debug == 'True':
        http_service.debug = True
    return cast(HttpService, http_service)


Res = Union[Dict[str, Any], str]
RouteFn = TypeVar("RouteFn", bound=Callable[..., Response])


def respond(
        err: Optional[str],
        res: Optional[Res] = None,
        headers: Optional[Dict[str, Any]] = None,
        status_code: int = 500,
) -> Response:
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


def not_found() -> Response:
    body = json.dumps({"Error": "Not Found"}).encode("utf8")
    return Response(
        status_code=404,
        body=body,
        headers={
            "Access-Control-Allow-Origin": "*",
            "Content-Type": "application/json",
            "Access-Control-Allow-Credentials": "false",
            "Access-Control-Allow-Methods": "GET,OPTIONS",
            "X-Requested-With": "*",
            "Access-Control-Allow-Headers": "Content-Type, Access-Control-Allow-Headers, Authorization, X-Requested-With",
        },
    )
