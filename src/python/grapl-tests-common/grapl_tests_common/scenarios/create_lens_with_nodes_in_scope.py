import hypothesis.strategies as st

import unittest
import hypothesis
from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.prelude import LensView, AssetView
from grapl_analyzerlib.node_types import PropType
from grapl_analyzerlib.test_utils.dgraph_utils import create_edge
from grapl_analyzerlib.test_utils.strategies.asset_view_strategy import get_or_create_asset, asset_props, AssetProps
from grapl_analyzerlib.test_utils.dgraph_utils import node_key_for_test, upsert

@hypothesis.given(asset_props())
def create_lens_with_nodes_in_scope(
    test: unittest.TestCase,
    graph_client: GraphClient,
    asset_prop: AssetProps,
) -> None:
    lens_name="whatever"  # TODO
    lens = LensView.get_or_create(
        gclient=graph_client,
        lens_name=lens_name,
        lens_type="engagement",
    )
    asset = get_or_create_asset(test, graph_client, asset_prop)
    create_edge(graph_client, from_uid=lens.uid, to_uid=asset.uid, edge_name="scope")