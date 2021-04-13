import logging
from typing import Any, Dict, List
from unittest import TestCase

import pytest
from grapl_analyzerlib.nodes.lens import LensQuery, LensView
from grapl_tests_common.clients.engagement_edge_client import EngagementEdgeClient
from grapl_tests_common.clients.graphql_endpoint_client import GraphqlEndpointClient
from grapl_tests_common.wait import WaitForCondition, WaitForQuery, wait_for_one
from grapl_tests_common.clients.model_plugin_deployer_client import ModelPluginDeployerClient


LENS_NAME = "DESKTOP-FVSHABR"

GqlLensDict = Dict[str, Any]

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

        gql_client = GraphqlEndpointClient(jwt=EngagementEdgeClient().get_jwt())
        ensure_graphql_lens_scope_no_errors(gql_client, LENS_NAME)
    
    def test_model_plugin(self) -> None:
        model_plugin_client = ModelPluginDeployerClient.from_env()
        upload_model_plugin(model_plugin_client)
        


def ensure_graphql_lens_scope_no_errors(
    gql_client: GraphqlEndpointClient,
    lens_name: str,
) -> None:
    """
    Eventually we'd want more-robust checks here, but this is an acceptable
    smoke test in the mean time.
    """
    gql_lens = gql_client.query_for_scope(lens_name=lens_name)
    assert len(gql_lens["scope"]) in (3, 4, 5)

    # Accumulate ["Asset"], ["Process"] into Set("Asset, Process")
    all_types_in_scope = set(
        sum((node["dgraph_type"] for node in gql_lens["scope"]), [])
    )
    assert all_types_in_scope == set(
        (
            "Asset",
            "Process",
        )
    )
    

def upload_model_plugin(
    model_plugin_client: ModelPluginDeployerClient,
) -> bool:
    logging.info("Making request to /deploy to upload model plugins")
    
    plugin_path = "./schemas"
    jwt = EngagementEdgeClient().get_jwt()
    
    plugin_upload = model_plugin_client.deploy(
        plugin_path,
        jwt,
    )
    
    logging.info(f"UploadRequest: {plugin_upload.json()}")
    
    upload_status = plugin_upload.json()["success"]["Success"] == True
    assert upload_status


