from http import HTTPStatus

import requests
from grapl_common.logger import get_structlogger
from grapl_common.test_user_creds import get_test_user_creds
from grapl_tests_common.clients.common import endpoint_url

_JSON_CONTENT_TYPE_HEADERS = {"Content-type": "application/json"}

LOGGER = get_structlogger()


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
            f"{self.endpoint}/api/auth/sign_in_with_password",
            json={
                "username": username,
                "password": password,
            },
            headers=_JSON_CONTENT_TYPE_HEADERS,
        )
        if resp.status_code != HTTPStatus.OK:
            raise GraplWebClientException(f"{resp.status_code}: {resp.text}")
        cookie: str | None = resp.cookies.get("actix-session")
        if not cookie:
            raise GraplWebClientException(
                f"Couldn't find actix-session cookie in {resp.cookies}"
            )
        return cookie

    def no_content_type(self) -> requests.Response:
        username, password = get_test_user_creds()

        resp = requests.post(
            f"{self.endpoint}/api/auth/sign_in_with_password",
            json={
                "username": username,
                "password": password,
            },
            # Explicitly no _JSON_CONTENT_TYPE_HEADERS
            headers={},
        )
        return resp
