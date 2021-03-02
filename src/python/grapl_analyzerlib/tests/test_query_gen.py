import unittest
import pytest
import hypothesis

from grapl_analyzerlib.prelude import GraphClient
from grapl_analyzerlib.nodes.lens import LensView, LensQuery


@pytest.mark.integration_test
class TestQueryGen(unittest.TestCase):
    @hypothesis.settings(deadline=None)
    @hypothesis.given(
        lens_name=hypothesis.strategies.text(),
    )
    def test_weird_chars_in_lens_name(self, lens_name: str) -> None:
        """
        Roundabout way to ensure some basic properties of filter generation.
        """
        client = GraphClient()
        lens = LensView.get_or_create(
            gclient=client,
            lens_name=lens_name,
            lens_type="engagement",
        )
        requery_lens = LensQuery().with_lens_name(lens_name).query_first(client)
        assert requery_lens.get_lens_name() == lens_name
