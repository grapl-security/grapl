import logging
import os
from http import HTTPStatus
from typing import Optional
from urllib.error import HTTPError

import requests

_JSON_CONTENT_TYPE_HEADERS = {"Content-type": "application/json"}
_ORIGIN = {
    "Origin": "https://local-grapl-engagement-ux-bucket.s3.amazonaws.com",
}


class EngagementEdgeException(Exception):
    pass


class EngagementEdgeClient:
    def __init__(self) -> None:
        hostname = os.environ["GRAPL_API_HOST"]
        port = os.environ["GRAPL_HTTP_FRONTEND_PORT"]
        self.endpoint = f"http://{hostname}:{port}/auth"

    def get_jwt(self) -> str:
        resp = requests.post(
            f"{self.endpoint}/login",
            json={
                # hardcoded when IS_LOCAL
                "username": "grapluser",
                # sha'd and pepper'd - see engagement view Login.tsx
                "password": "2ae5ddfb1eeeed45d502bcfd0c7b8f962f24bf85328ba942f32a31c0229c295a",
            },
            # TODO: Should consume the deployment name instead of hardcoded.
            headers={
                **_JSON_CONTENT_TYPE_HEADERS,
                **_ORIGIN,
            },
        )
        if resp.status_code != HTTPStatus.OK:
            raise EngagementEdgeException(f"{resp.status_code}: {resp.text}")
        cookie: Optional[str] = resp.cookies.get("grapl_jwt")
        if not cookie:
            raise EngagementEdgeException(
                f"Couldn't find grapl_jwt cookie in {resp.cookies}"
            )
        return cookie

    def invalid_creds(self) -> requests.Response:
        resp = requests.post(
            f"{self.endpoint}/login",
            json={
                "username": "fakeuser",
                "password": "fakepassword",
            },
            headers={
                **_JSON_CONTENT_TYPE_HEADERS,
                **_ORIGIN,
            },
        )
        return resp
    
    def empty_creds(self) -> requests.Response:
        resp = requests.post(
            f"{self.endpoint}/login",
            json={
                "username": "",
                "password": "",
            },
            headers={
                **_JSON_CONTENT_TYPE_HEADERS,
                **_ORIGIN,
            },
        )
        return resp

    def get_notebook(self, jwt: str) -> str:
        cookies = {"grapl_jwt": jwt}
        resp = requests.post(
            f"{self.endpoint}/getNotebook",
            cookies=cookies,
            headers=_ORIGIN,
        )
        url: str = resp.json()["success"]["notebook_url"]
        return url
