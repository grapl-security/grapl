import unittest
import pytest

from grapl_analyzerlib.prelude import GraphClient
from grapl_analyzerlib.nodes.engagement import EngagementView


@pytest.mark.integration_test
class TestEngagement(unittest.TestCase):
    def test_get_or_create(self) -> None:
        client = GraphClient()
        engagement = EngagementView.get_or_create(eg_client=client, lens_name="test")
