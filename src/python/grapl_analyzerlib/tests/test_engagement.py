import unittest
import pytest
from hypothesis import given

from grapl_analyzerlib.prelude import GraphClient
from grapl_analyzerlib.nodes.engagement import EngagementView
from test_utils.dgraph_utils import upsert, create_edge

@pytest.mark.integration_test
class TestEngagement(unittest.TestCase):
    def test_get_or_create(self) -> None:
        client = GraphClient()
        engagement = EngagementView.get_or_create(eg_client=client, lens_name="test")