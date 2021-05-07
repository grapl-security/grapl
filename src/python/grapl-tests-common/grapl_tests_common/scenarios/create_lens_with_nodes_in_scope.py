import unittest

from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.prelude import LensView
from grapl_analyzerlib.test_utils.dgraph_utils import create_edge, random_key_for_test
from grapl_analyzerlib.test_utils.strategies.asset_view_strategy import (
    AssetProps,
    get_or_create_asset,
)


def create_lens_with_nodes_in_scope(
    test: unittest.TestCase,
    graph_client: GraphClient,
    asset_props: AssetProps,
) -> LensView:
    lens_name = random_key_for_test(test)  #  just something tied to this test + random
    lens = LensView.get_or_create(
        gclient=graph_client,
        lens_name=lens_name,
        lens_type="engagement",
    )

    asset = get_or_create_asset(test, graph_client, asset_props)
    create_edge(graph_client, from_uid=lens.uid, to_uid=asset.uid, edge_name="scope")
    return lens
