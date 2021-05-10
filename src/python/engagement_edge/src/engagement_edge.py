from __future__ import annotations

import json
from typing import (
    Any,
    Optional,
)

from chalice import Response
from src.lib.env_vars import DEPLOYMENT_NAME
from src.lib.sagemaker import create_sagemaker_client

from grapl_common.debugger.vsc_debugger import wait_for_vsc_debugger
from grapl_common.grapl_logger import get_module_grapl_logger
from grapl_http_service.auth import login_cookie, no_auth, requires_auth, check_jwt
from grapl_http_service.grapl_http_service import init, respond

LOGGER = get_module_grapl_logger()

wait_for_vsc_debugger(service="engagement_edge")

app = init(app_name="engagement-edge")


def lambda_login(event: Any) -> Optional[str]:
    body = json.loads(
        event.raw_body.decode()
    )  # 'json_body' is a more natural choice, but has issues:  c.f. github issue aws/chalice#1188
    cookie = login_cookie(body["username"], body["password"])
    # Clear out the password from the dict, to avoid accidentally logging it
    body["password"] = ""
    if cookie:
        return cookie
    return None


@no_auth(app, "/login")
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


@no_auth(app, "/checkLogin")
def check_login() -> Response:
    LOGGER.debug(f"/checkLogin {app.current_request}")
    request = app.current_request

    if check_jwt(request.headers):
        return respond(None, "True")
    else:
        return respond(None, "False")


@requires_auth(app, "/getNotebook")
def get_notebook() -> Response:
    # cross-reference with `engagement.ts` notebookInstanceName
    notebook_name = f"{DEPLOYMENT_NAME}-Notebook"
    client = create_sagemaker_client()
    url = client.get_presigned_url(notebook_name)
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
