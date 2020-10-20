import requests
from typing import Optional
from http import HTTPStatus

_JSON_CONTENT_TYPE_HEADERS = {"Content-type": "application/json"}


class EngagementEdgeException(Exception):
    pass


class EngagementEdgeClient:
    def __init__(self, use_docker_links: bool = False) -> None:
        hostname = "grapl-engagement-edge" if use_docker_links else "localhost"
        self.endpoint = f"http://{hostname}:8900"

    def get_jwt(self) -> str:
        resp = requests.post(
            f"{self.endpoint}/login",
            json={
                # hardcoded when IS_LOCAL
                "username": "grapluser",
                # sha'd and pepper'd - see engagement view Login.tsx
                "password": "2ae5ddfb1eeeed45d502bcfd0c7b8f962f24bf85328ba942f32a31c0229c295a",
            },
            headers=_JSON_CONTENT_TYPE_HEADERS,
        )
        if resp.status_code != HTTPStatus.OK:
            raise EngagementEdgeException(f"{resp.status_code}: {resp.text}")
        cookie: Optional[str] = resp.cookies.get("grapl_jwt")
        if not cookie:
            raise EngagementEdgeException(
                f"Couldn't find grapl_jwt cookie in {resp.cookies}"
            )
        return cookie
