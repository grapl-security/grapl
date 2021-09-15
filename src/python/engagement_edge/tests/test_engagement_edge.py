import unittest

import pytest
from chalice.test import Client
from engagement_edge.engagement_edge import JWT_SECRET, app
from grapl_tests_common.clients.engagement_edge_client import EngagementEdgeClient
from grapl_tests_common.clients.grapl_web_client import GraplWebClient

# gross hack because engagement edge is pseudo singleton
JWT_SECRET.secret = "hey im a fake secret"

# TODO: These tests will fail at the pytest collection stage if
# DEPLOYMENT_NAME isn't in the environment because of how env_vars.py is
# currently written


class TestEngagementEdgeChalice(unittest.TestCase):
    # Unit, not integration, for the record.
    def test_requires_auth_fails_without_cookie_headers(self) -> None:
        with Client(app) as client:
            result = client.http.post(
                "/getNotebook",
            )
        assert result.status_code == 403
        assert result.json_body == {
            "error": "Must log in: No grapl_jwt cookie supplied."
        }

    def test_requires_auth_fails_with_incorrect_cookie_headers(self) -> None:
        with Client(app) as client:
            result = client.http.post(
                "/getNotebook", headers={"Cookie": "grapl_jwt=im-not-a-jwt"}
            )
        assert result.status_code == 403
        assert result.json_body == {
            "error": "Must log in: Could not decode grapl_jwt cookie."
        }


@pytest.mark.integration_test
class TestEngagementEdgeClient(unittest.TestCase):
    def test_get_notebook_link(self) -> None:
        client = GraplWebClient()
        jwt = client.get_actix_session()
