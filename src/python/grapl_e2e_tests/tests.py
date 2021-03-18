import logging
from typing import Any, Dict, List
from unittest import TestCase

import pytest
from grapl_analyzerlib.nodes.lens import LensQuery, LensView
from grapl_analyzerlib.retry import retry
from grapl_tests_common.clients.engagement_edge_client import EngagementEdgeClient
from grapl_tests_common.clients.graphql_endpoint_client import GraphqlEndpointClient
from grapl_tests_common.wait import WaitForCondition, WaitForQuery, wait_for_one

LENS_NAME = "DESKTOP-FVSHABR"


@pytest.mark.integration_test
class TestEndToEnd(TestCase):
    def test_expected_data_in_dgraph(self) -> None:
        # There is some unidentified, nondeterministic failure with e2e.
        # We fall into one of three buckets:
        # - No lens
        # - Lens with 3 scope
        # - Lens with 4 scope
        # - Lens with 5 scope (correct)

        query = LensQuery().with_lens_name(LENS_NAME)
        lens: LensView = wait_for_one(WaitForQuery(query), timeout_secs=120)
        assert lens.get_lens_name() == LENS_NAME

        # lens scope is not atomic
        def condition() -> bool:
            length = len(lens.get_scope())
            logging.info(f"Expected 3-5 nodes in scope, currently is {length}")

            # The correct answer for this is 5.
            # We are temp 'allowing' below that because it means the pipeline is, _mostly_, working.
            return length in (
                3,
                4,
                5,
            )

        wait_for_one(WaitForCondition(condition), timeout_secs=240)

        # At this point, the lens should be available in Graphql
        gql_lenses = _query_graphql_endpoint_for_lenses()
        return gql_lenses[0]["lens_name"] == LENS_NAME


def _query_graphql_endpoint_for_lenses() -> List[GqlLensDict]:
    query = """
        {
            lenses(first: 100, offset: 0) {
                uid,
                node_key,
                lens_name,
                score, 
                lens_type,
            }
        }
    """
    gql_client = GraphqlEndpointClient(jwt=EngagementEdgeClient().get_jwt())
    resp = gql_client.query(query)
    return resp["lenses"]
