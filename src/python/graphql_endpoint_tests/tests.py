from datetime import timedelta
from typing import Any, Dict, List
from unittest import TestCase

import hypothesis
import pytest
from grapl_analyzerlib.test_utils.strategies.asset_view_strategy import (
    AssetProps,
    asset_props_strategy,
)
from grapl_common.debugger.vsc_debugger import wait_for_vsc_debugger
from grapl_common.grapl_logger import get_module_grapl_logger
from grapl_tests_common.clients.engagement_edge_client import EngagementEdgeClient
from grapl_tests_common.clients.graphql_endpoint_client import GraphqlEndpointClient
from grapl_tests_common.scenarios.create_lens_with_nodes_in_scope import *

LOGGER = get_module_grapl_logger()

GqlLensDict = Dict[str, Any]

wait_for_vsc_debugger(service="graphql_endpoint_tests")


@pytest.mark.integration_test
class TestGraphqlEndpoint(TestCase):
    @hypothesis.given(
        asset_props=asset_props_strategy(),
    )
    @hypothesis.settings(deadline=timedelta(seconds=10))
    def test_create_lens_shows_up_in_graphql(
        self,
        asset_props: AssetProps,
    ) -> None:
        graph_client = GraphClient()
        graphql_client = GraphqlEndpointClient(jwt=EngagementEdgeClient().get_jwt())

        lens = create_lens_with_nodes_in_scope(self, graph_client, asset_props)
        lens_name = lens.get_lens_name()
        assert lens_name

        # Check that this lens shows up in the "show all lenses" view
        # NOTE: This test could be somewhat finicky, since it just gets the first 1000 lenses
        gql_lenses = _query_graphql_endpoint_for_lenses(graphql_client)
        assert lens_name in [l["lens_name"] for l in gql_lenses]

        # Query by that lens name
        gql_lens = _query_graphql_endpoint_for_scope(lens_name, graphql_client)
        # For some reason, upon create, `lens.uid` comes back as a string like "0x5"
        assert gql_lens["uid"] == int(lens.uid, 0)  # type: ignore
        assert gql_lens["lens_name"] == lens_name
        # Check the nodes in scope
        assert len(gql_lens["scope"]) == 1
        assert gql_lens["scope"][0]["hostname"] == asset_props["hostname"]


def _query_graphql_endpoint_for_lenses(
    gql_client: GraphqlEndpointClient,
) -> List[GqlLensDict]:
    # Just get *all* lenses
    query = """
    {
        lenses(first: 1000, offset: 0) {
            uid,
            node_key,
            lens_name,
            score, 
            lens_type,
        }
    }
    """
    resp = gql_client.query(query)
    return resp["lenses"]


def _query_graphql_endpoint_for_scope(
    lens_name: str, graphql_client: GraphqlEndpointClient
) -> GqlLensDict:
    # This query is based off the lens_scope query in /expandLensScopeQuery.tsx

    query = """
    query LensScopeByName($lens_name: String!) {
        lens_scope(lens_name: $lens_name) {
            uid,
            node_key,
            lens_type,
            lens_name,
            dgraph_type,
            score,
            scope {
                ... on Asset {
                    uid, 
                    node_key, 
                    dgraph_type,
                    hostname,
                    asset_ip{
                        ip_address
                    }, 
                    asset_processes{
                        uid, 
                        node_key, 
                        dgraph_type,
                        process_name, 
                        process_id,
                    },
                    files_on_asset{
                        uid, 
                        node_key, 
                        dgraph_type,
                        file_path
                    }, 
                    risks {  
                        uid,
                        dgraph_type,
                        node_key, 
                        analyzer_name, 
                        risk_score
                    },
                }
            }
        }
    }
    """
    resp = graphql_client.query(query, {"lens_name": lens_name})
    return resp["lens_scope"]
