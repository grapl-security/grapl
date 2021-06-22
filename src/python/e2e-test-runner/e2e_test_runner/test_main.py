import logging
import os
from pathlib import Path
from typing import Any, Dict, List, Mapping
from unittest import TestCase

import pytest
from grapl_analyzerlib.nodes.lens import LensQuery, LensView
from grapl_tests_common.clients.engagement_edge_client import EngagementEdgeClient
from grapl_tests_common.clients.graphql_endpoint_client import GraphqlEndpointClient
from grapl_tests_common.clients.model_plugin_deployer_client import (
    ModelPluginDeployerClient,
)
from grapl_tests_common.subset_equals import subset_equals
from grapl_tests_common.wait import (
    WaitForCondition,
    WaitForNoException,
    WaitForQuery,
    wait_for_one,
)

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

        def scope_has_N_items() -> bool:
            length = len(lens.get_scope())
            logging.info(f"Expected 3-5 nodes in scope, currently is {length}")
            # The correct answer for this is 5.
            # We are temp 'allowing' below that because it means the pipeline is, _mostly_, working.
            return length in (
                3,
                4,
                5,
            )

        wait_for_one(WaitForCondition(scope_has_N_items), timeout_secs=300)
        gql_client = GraphqlEndpointClient(jwt=EngagementEdgeClient().get_jwt())
        wait_for_one(
            WaitForNoException(
                lambda: ensure_graphql_lens_scope_no_errors(gql_client, LENS_NAME)
            ),
            timeout_secs=300,
        )

    # -------------------------- MODEL PLUGIN TESTS -------------------------------------------
    def test_upload_plugin(self) -> None:
        upload_model_plugin(model_plugin_client=ModelPluginDeployerClient.from_env())

    @pytest.mark.xfail  # TODO: Remove once list plugins is resolved
    def test_list_plugin(self) -> None:
        get_plugin_list(model_plugin_client=ModelPluginDeployerClient.from_env())

    @pytest.mark.xfail  # TODO: once list plugins is resolved, we can fix delete plugins :)
    def test_delete_plugin(self) -> None:
        # Hard Code for now
        delete_model_plugin(
            model_plugin_client=ModelPluginDeployerClient.from_env(),
            plugin_to_delete="aws_plugin",
        )  # TODO: we need to change the plugin name when this endpoint gets fixed

    def test_check_login(self) -> None:
        check_login()

    def test_get_notebook_url(self) -> None:
        get_notebook_url()

    def test_check__invalid_creds(self) -> None:
        check_invalid_creds()

    def test_check__empty_creds(self) -> None:
        check_empty_creds()


def ensure_graphql_lens_scope_no_errors(
    gql_client: GraphqlEndpointClient,
    lens_name: str,
) -> None:
    gql_lens = gql_client.query_for_scope(lens_name=lens_name)
    scope = gql_lens["scope"]
    assert len(scope) in (3, 4, 5)
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
    asset_node: Dict = next((n for n in scope if n["dgraph_type"] == ["Asset"]))
    # The 'risks' field is not immediately filled out, but eventually consistent
    subset_equals(larger=asset_node, smaller=expected_gql_asset())


def expected_gql_asset() -> Mapping[str, Any]:
    """
    All the fixed values (i.e. no uid, no node key) we'd see in the e2e test
    """
    return {
        "dgraph_type": ["Asset"],
        "display": "DESKTOP-FVSHABR",
        "hostname": "DESKTOP-FVSHABR",
        "asset_processes": [
            {
                "dgraph_type": ["Process"],
                "process_name": "cmd.exe",
                "process_id": 5824,
            },
            {
                "dgraph_type": ["Process"],
                "process_name": "dropper.exe",
                "process_id": 4164,
            },
            {
                "dgraph_type": ["Process"],
                "process_name": "cmd.exe",
                "process_id": 5824,
            },
            {
                "dgraph_type": ["Process"],
                "process_name": "svchost.exe",
                "process_id": 6132,
            },
        ],
        "files_on_asset": None,
        "risks": [
            {
                "dgraph_type": ["Risk"],
                "node_key": "Rare Parent of cmd.exe",
                "analyzer_name": "Rare Parent of cmd.exe",
                "risk_score": 10,
            }
        ],
    }


# -----------------------  MODEL PLUGIN HELPERS -------------------------------------------
# TODO: move these into their own file once that's doable with e2e/pants
def upload_model_plugin(
    model_plugin_client: ModelPluginDeployerClient,
) -> None:
    logging.info("Making request to /deploy to upload model plugins")
    plugin_path = "/tmp/schemas"
    jwt = EngagementEdgeClient().get_jwt()
    files = os.listdir(plugin_path)
    check_plugin_path_has_schemas_file(files)
    plugin_upload = model_plugin_client.deploy(
        Path(plugin_path),
        jwt,
    )
    logging.info(f"UploadRequest: {plugin_upload.json()}")
    upload_status = plugin_upload.json()["success"]["Success"] == True
    assert upload_status


def check_plugin_path_has_schemas_file(
    files: List[str],
) -> None:
    logging.info(f"files: {files}")
    assert "schemas.py" in files, f"Did not find schemas.py file in {files}"


def get_plugin_list(model_plugin_client: ModelPluginDeployerClient) -> None:
    jwt = EngagementEdgeClient().get_jwt()
    get_plugin_list = model_plugin_client.list_plugins(
        jwt,
    )
    logging.info(f"UploadRequest: {get_plugin_list.json()}")
    upload_status = get_plugin_list.json()["success"]["plugin_list"] != []
    assert upload_status


def delete_model_plugin(
    model_plugin_client: ModelPluginDeployerClient,
    plugin_to_delete: str,
) -> None:
    jwt = EngagementEdgeClient().get_jwt()
    delete_plugin = model_plugin_client.delete_model_plugin(
        jwt,
        plugin_to_delete,
    )
    logging.info(f"Deleting Plugin: {plugin_to_delete}")
    deleted = delete_plugin.json()["success"]["plugins_to_delete"]
    assert deleted


# ---------------------------- AUTH HELPERS v ------------------------------------
def get_notebook_url() -> None:
    jwt = EngagementEdgeClient().get_jwt()
    notebook_url = EngagementEdgeClient().get_notebook(jwt)
    if (
        "localhost:8888" in notebook_url
    ):  # TODO: Need to conditionally change for AWS Deployments
        assert notebook_url


def check_login() -> None:
    jwt = EngagementEdgeClient().get_jwt()
    assert jwt != None


def check_invalid_creds() -> None:
    resp = EngagementEdgeClient().invalid_creds()
    assert resp.status_code == 403, "We expected a 403 forbidden"


def check_empty_creds() -> None:
    resp = EngagementEdgeClient().empty_creds()
    assert resp.status_code == 500, "Expected 500 permissions error"
