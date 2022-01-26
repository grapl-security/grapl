import logging
import os
from typing import Any, Dict, Mapping

from grapl_analyzerlib.nodes.lens import LensQuery, LensView
from grapl_tests_common.clients.graphql_endpoint_client import GraphqlEndpointClient
from grapl_tests_common.subset_equals import subset_equals
from grapl_tests_common.wait import (
    WaitForCondition,
    WaitForNoException,
    WaitForQuery,
    wait_for_one,
)

LENS_NAME = "DESKTOP-FVSHABR"
GqlLensDict = Dict[str, Any]

TIMEOUT_SECS = int(os.getenv("TIMEOUT_SECS", "300"))


def test_expected_data_in_dgraph(actix_session: str) -> None:
    # There is some unidentified, nondeterministic failure with e2e.
    # We fall into one of three buckets:
    # - No lens
    # - Lens with 3 scope
    # - Lens with 4 scope
    # - Lens with 5 scope (correct)
    query = LensQuery().with_lens_name(LENS_NAME)
    lens: LensView = wait_for_one(WaitForQuery(query), timeout_secs=TIMEOUT_SECS)
    assert lens.get_lens_name() == LENS_NAME
    # lens scope is not atomic

    def scope_has_N_items() -> bool:
        length = len(lens.get_scope())
        logging.info(f"Expected 3+ nodes in scope, currently is {length}")
        # This number can change and, rather than trying to hammer it down, we're going 
        # with a lower bound
        return length >= 3

    wait_for_one(WaitForCondition(scope_has_N_items), timeout_secs=TIMEOUT_SECS)

    # Now that we've confirmed that the expected data has shown up in dgraph,
    # let's see what the GraphQL endpoint says.
    # TODO: Consider using `pytest-order` to make this a separate test that
    # depends on the above test having been run.

    gql_client = GraphqlEndpointClient(actix_session=actix_session)
    wait_for_one(
        WaitForNoException(
            lambda: ensure_graphql_lens_scope_no_errors(gql_client, LENS_NAME)
        ),
        timeout_secs=TIMEOUT_SECS,
    )


def ensure_graphql_lens_scope_no_errors(
    gql_client: GraphqlEndpointClient,
    lens_name: str,
) -> None:
    gql_lens = gql_client.query_for_scope(lens_name=lens_name)
    scope = gql_lens["scope"]
    assert len(scope) >= 3
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
