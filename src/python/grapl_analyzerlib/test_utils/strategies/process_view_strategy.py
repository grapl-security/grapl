import unittest
from typing import NewType, Dict, cast

import hypothesis.strategies as st
from pydgraph import DgraphClient

from grapl_analyzerlib.nodes.process_node import ProcessView
from grapl_analyzerlib.nodes.types import Property
from test_utils.dgraph_utils import node_key_for_test, upsert

ProcessProps = NewType("ProcessProps", Dict[str, Property])


def process_props() -> st.SearchStrategy[ProcessProps]:
    return st.builds(
        ProcessProps,
        st.builds(
            dict,
            node_key=st.uuids(),
            process_id=st.integers(min_value=1, max_value=2 ** 32),
            created_timestamp=st.integers(min_value=0, max_value=2 ** 48),
            terminate_time=st.integers(min_value=0, max_value=2 ** 48),
            image_name=st.text(),
            process_name=st.text(),
            arguments=st.text(),
        ),
    )


def get_or_create_process(
    test: unittest.TestCase, local_client: DgraphClient, node_props: ProcessProps
) -> ProcessView:
    node_key = node_key_for_test(test, str(node_props["node_key"]))
    return cast(
        ProcessView, upsert(local_client, "Process", ProcessView, node_key, node_props)
    )
