import unittest
import uuid

from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.node_types import PropType
from grapl_analyzerlib.prelude import AssetView, LensView
from grapl_analyzerlib.test_utils.dgraph_utils import create_edge
from grapl_analyzerlib.test_utils.strategies.asset_view_strategy import (
    AssetProps,
    get_or_create_asset,
)


def create_lens_with_nodes_in_scope(
    test: unittest.TestCase,
    graph_client: GraphClient,
    asset_props: AssetProps,
) -> LensView:
    # Give each test run its own lens
    lens_name = f"{test.id()}-{uuid.uuid4()}"
    lens = LensView.get_or_create(
        gclient=graph_client,
        lens_name=lens_name,
        lens_type="engagement",
    )

    # Each test run should have exactly one asset and one lens
    asset_props["hostname"] = f"{lens_name}-{asset_props['hostname']}"
    asset = get_or_create_asset(test, graph_client, asset_props)
    create_edge(graph_client, from_uid=lens.uid, to_uid=asset.uid, edge_name="scope")
    return lens
