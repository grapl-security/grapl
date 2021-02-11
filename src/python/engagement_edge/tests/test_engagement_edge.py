import unittest

import pytest
from chalice.test import Client
from grapl_tests_common.clients.engagement_edge_client import EngagementEdgeClient
from src.engagement_edge import JWT_SECRET, app

# gross hack because engagement edge is pseudo singleton
JWT_SECRET.secret = "hey im a fake secret"


@pytest.mark.integration_test
class TestEngagementEdgeClient(unittest.TestCase):
    def test_get_notebook_link(self) -> None:
        client = EngagementEdgeClient(use_docker_links=True)
        jwt = client.get_jwt()
        notebook_url = client.get_notebook(jwt=jwt)
        assert "localhost:8888" in notebook_url
