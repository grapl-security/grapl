from http import HTTPStatus
from typing import Optional

import requests
from grapl_common.grapl_logger import get_module_grapl_logger
from grapl_common.test_user_creds import get_test_user_creds
from grapl_tests_common.clients.common import endpoint_url

_JSON_CONTENT_TYPE_HEADERS = {"Content-type": "application/json"}

LOGGER = get_module_grapl_logger(default_log_level="DEBUG")


class GraplWebClientException(Exception):
    pass


class GraplWebClient:
    def __init__(self) -> None:
        self.endpoint = endpoint_url(suffix="")
        LOGGER.debug(f"created GraplWebClient for endpoint {self.endpoint}")

    def get_actix_session(self) -> str:
        LOGGER.debug("retrieving actix cookie")
        username, password = get_test_user_creds()

        resp = requests.post(
            f"{self.endpoint}/auth/login",
            json={
                "username": username,
                "password": password,
            },
            headers=_JSON_CONTENT_TYPE_HEADERS,
        )
        if resp.status_code != HTTPStatus.OK:
            raise GraplWebClientException(f"{resp.status_code}: {resp.text}")
        cookie: Optional[str] = resp.cookies.get("actix-session")
        if not cookie:
            raise GraplWebClientException(
                f"Couldn't find actix-session cookie in {resp.cookies}"
            )
        return cookie

    def real_user_fake_password(self) -> requests.Response:
        username, _ = get_test_user_creds()
        resp = requests.post(
            f"{self.endpoint}/auth/login",
            json={
                "username": username,
                "password": "fakepassword",
            },
            headers=_JSON_CONTENT_TYPE_HEADERS,
        )
        return resp

    def nonexistent_user(self) -> requests.Response:
        resp = requests.post(
            f"{self.endpoint}/auth/login",
            json={
                "username": "fakeuser",
                "password": "fakepassword",
            },
            headers=_JSON_CONTENT_TYPE_HEADERS,
        )
        return resp

    def empty_creds(self) -> requests.Response:
        resp = requests.post(
            f"{self.endpoint}/auth/login",
            json={
                "username": "",
                "password": "",
            },
            headers=_JSON_CONTENT_TYPE_HEADERS,
        )
        return resp

    def no_content_type(self) -> requests.Response:
        username, password = get_test_user_creds()

        resp = requests.post(
            f"{self.endpoint}/auth/login",
            json={
                "username": username,
                "password": password,
            },
            # Explicitly no _JSON_CONTENT_TYPE_HEADERS
            headers={},
        )
        return resp
