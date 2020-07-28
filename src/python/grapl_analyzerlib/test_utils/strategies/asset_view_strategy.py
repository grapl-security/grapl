import unittest
from typing import NewType, Dict, cast

import hypothesis.strategies as st
from pydgraph import DgraphClient

from grapl_analyzerlib.nodes.asset_node import AssetView
from grapl_analyzerlib.nodes.types import Property
from test_utils.dgraph_utils import node_key_for_test, upsert

AssetProps = NewType("AssetProps", Dict[str, Property])


def asset_props() -> st.SearchStrategy[AssetProps]:
    return st.builds(
        AssetProps, st.builds(dict, node_key=st.uuids(), hostname=st.text(),)
    )


def get_or_create_asset(
    test: unittest.TestCase, local_client: DgraphClient, node_props: AssetProps
) -> AssetView:
    node_key = node_key_for_test(test, str(node_props["node_key"]))
    return cast(
        AssetView, upsert(local_client, "Asset", AssetView, node_key, node_props)
    )
