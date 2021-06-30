import os
from http import HTTPStatus
from typing import Optional

import boto3
import requests
from grapl_common.env_helpers import SecretsManagerClientFactory
from grapl_common.grapl_logger import get_module_grapl_logger
from grapl_tests_common.clients.common import endpoint_url

_JSON_CONTENT_TYPE_HEADERS = {"Content-type": "application/json"}
_ORIGIN = {
    "Origin": "https://local-grapl-engagement-ux-bucket.s3.amazonaws.com",
}

LOGGER = get_module_grapl_logger(default_log_level="DEBUG")


class EngagementEdgeException(Exception):
    pass


def _get_test_user_password(deployment_name: str) -> str:
    secretsmanager = SecretsManagerClientFactory(boto3).from_env()
    LOGGER.debug(f"retrieving {deployment_name}-TestUserPassword")
    return secretsmanager.get_secret_value(
        SecretId=f"{deployment_name}-TestUserPassword"
    )["SecretString"]


class EngagementEdgeClient:
    def __init__(self) -> None:
        self.endpoint = endpoint_url("/auth")
        LOGGER.debug(f"created EngagementEdgeClient for endpoint {self.endpoint}")

    def get_jwt(self) -> str:
        LOGGER.debug("retrieving jwt cookie")
        username = os.environ["GRAPL_TEST_USER_NAME"]
        password = _get_test_user_password(
            deployment_name=os.environ["DEPLOYMENT_NAME"]
        )
        LOGGER.debug(f"logging in with user {username} at {self.endpoint}/login")
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
        LOGGER.debug("retrieved jwt cookie")
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
