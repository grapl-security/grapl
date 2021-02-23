import unittest
from typing import NewType, Dict, cast

import hypothesis.strategies as st

from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.nodes.asset import AssetView
from grapl_analyzerlib.node_types import PropType
from test_utils.dgraph_utils import node_key_for_test, upsert

AssetProps = NewType("AssetProps", Dict[str, PropType])


def asset_props() -> st.SearchStrategy[AssetProps]:
    return st.builds(
        AssetProps,
        st.builds(
            dict,
            node_key=st.uuids(),
            hostname=st.text(),
        ),
    )


def get_or_create_asset(
    test: unittest.TestCase, graph_client: GraphClient, node_props: AssetProps
) -> AssetView:
    node_key = node_key_for_test(test, str(node_props["node_key"]))
    return cast(
        AssetView, upsert(graph_client, "Asset", AssetView, node_key, node_props)
    )
