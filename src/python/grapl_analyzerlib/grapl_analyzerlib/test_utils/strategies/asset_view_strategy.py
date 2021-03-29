import unittest
from typing import cast, Dict, Any, NewType
from uuid import UUID

import hypothesis.strategies as st

from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.nodes.asset import AssetView
from grapl_analyzerlib.node_types import PropType
from grapl_analyzerlib.test_utils.dgraph_utils import node_key_for_test, upsert
from grapl_analyzerlib.test_utils.strategies.misc import text_dgraph_compat

AssetProps = NewType("AssetProps", Dict[str, Any])


def asset_props_strategy() -> st.SearchStrategy[AssetProps]:
    return st.builds(
        AssetProps,
        st.builds(
            dict,
            node_key=st.uuids(),
            hostname=text_dgraph_compat(),
        ),
    )


def get_or_create_asset(
    test: unittest.TestCase, graph_client: GraphClient, node_props: AssetProps
) -> AssetView:
    node_key = node_key_for_test(test, str(node_props["node_key"]))
    return cast(
        AssetView, upsert(graph_client, "Asset", AssetView, node_key, node_props)
    )
