import unittest

import pytest
from grapl_tests_common.clients.engagement_edge_client import EngagementEdgeClient
from src.engagement_edge import JWT_SECRET

# gross hack because engagement edge is pseudo singleton
JWT_SECRET.secret = "hey im a fake secret"

# TODO: These tests will fail at the pytest collection stage if
# DEPLOYMENT_NAME isn't in the environment because of how env_vars.py is
# currently written


@pytest.mark.integration_test
class TestEngagementEdgeClient(unittest.TestCase):
    def test_get_notebook_link(self) -> None:
        client = EngagementEdgeClient()
        jwt = client.get_jwt()
        notebook_url = client.get_notebook(jwt=jwt)
        assert "localhost:8888" in notebook_url
