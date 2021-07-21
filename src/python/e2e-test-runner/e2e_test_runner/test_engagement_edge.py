"""
TODO (wimax July 2020): I don't see anything in here that indicates that
screams "e2e test", this certainly seems like more of an integration test.

There's nothing here that does anything cross-service.
Perhaps it's just "does it work in AWS?"
"""
from unittest import TestCase

from grapl_tests_common.clients.engagement_edge_client import EngagementEdgeClient


class TestEngagementEdge(TestCase):
    def test_check_login(self) -> None:
        check_login()

    def test_get_notebook_url(self) -> None:
        get_notebook_url()

    def test_check__invalid_creds(self) -> None:
        check_invalid_creds()

    def test_check__empty_creds(self) -> None:
        check_empty_creds()


def get_notebook_url() -> None:
    jwt = EngagementEdgeClient().get_jwt()
    notebook_url = EngagementEdgeClient().get_notebook(jwt)
    if (
        "localhost:8888" in notebook_url
    ):  # TODO: Need to conditionally change for AWS Deployments
        assert notebook_url


def check_login() -> None:
    jwt = EngagementEdgeClient().get_jwt()
    assert jwt != None


def check_invalid_creds() -> None:
    resp = EngagementEdgeClient().invalid_creds()
    assert resp.status_code == 403, "We expected a 403 forbidden"


def check_empty_creds() -> None:
    resp = EngagementEdgeClient().empty_creds()
    assert resp.status_code == 500, "Expected 500 permissions error"
