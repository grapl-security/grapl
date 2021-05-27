import hashlib
import os
from http import HTTPStatus
from typing import Optional

import boto3
import requests
from grapl_common.env_helpers import SecretsManagerClientFactory

_JSON_CONTENT_TYPE_HEADERS = {"Content-type": "application/json"}
_ORIGIN = {
    "Origin": "https://local-grapl-engagement-ux-bucket.s3.amazonaws.com",
}


class EngagementEdgeException(Exception):
    pass


def _get_test_user_password(deployment_name: str) -> str:
    secretsmanager = SecretsManagerClientFactory(boto3).from_env()
    return secretsmanager.get_secret_value(
        SecretId=f"{deployment_name}-TestUserPassword"
    )["SecretString"]


def _sha_and_pepper(username: str, password: str) -> str:
    # see src/js/engagement_view/src/components/login/utils/passwordHashing.tsx
    pepper = "f1dafbdcab924862a198deaa5b6bae29aef7f2a442f841da975f1c515529d254"
    hashed = hashlib.sha256((password + pepper + username).encode("utf-8"))
    for _ in range(5000):
        hashed = hashlib.sha256(hashed.hexdigest().encode("utf-8"))
    return hashed.hexdigest()


class EngagementEdgeClient:
    def __init__(self) -> None:
        self.endpoint = f'http://{os.environ["GRAPL_API_HOST"]}/auth'

    def get_jwt(self) -> str:
        username = os.environ["GRAPL_TEST_USER_NAME"]
        password = _sha_and_pepper(
            username=username,
            password=_get_test_user_password(
                deployment_name=os.environ["DEPLOYMENT_NAME"]
            ),
        )
        resp = requests.post(
            f"{self.endpoint}/login",
            json={
                "username": username,
                "password": password,
            },
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
