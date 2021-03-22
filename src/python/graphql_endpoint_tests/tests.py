import logging
from typing import Any, Dict, List
from unittest import TestCase

import pytest
from grapl_tests_common.clients.engagement_edge_client import EngagementEdgeClient
from grapl_tests_common.clients.graphql_endpoint_client import GraphqlEndpointClient
from grapl_tests_common.scenarios.create_lens_with_nodes_in_scope import *

LENS_NAME = "DESKTOP-FVSHABR"

GqlLensDict = Dict[str, Any]


@pytest.mark.integration_test
class TestGraphqlEndpoint(TestCase):
    def test_create_lens_shows_up_in_graphql(self) -> None:
        # At this point, the lens should be available in Graphql
        graph_client = GraphClient()
        create_lens_with_nodes_in_scope(self, graph_client)
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
