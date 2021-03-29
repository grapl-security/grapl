import unittest
from typing import NewType, Dict, cast

import hypothesis.strategies as st

from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.nodes.process import ProcessView
from grapl_analyzerlib.node_types import PropType
from grapl_analyzerlib.test_utils.dgraph_utils import node_key_for_test, upsert

ProcessProps = NewType("ProcessProps", Dict[str, PropType])


def process_props() -> st.SearchStrategy[ProcessProps]:
    return st.builds(
        ProcessProps,
        st.builds(
            dict,
            node_key=st.uuids(),
            process_id=st.integers(min_value=1, max_value=2 ** 32),
            created_timestamp=st.integers(min_value=0, max_value=2 ** 48),
            terminate_time=st.integers(min_value=0, max_value=2 ** 48),
            image_name=st.text(min_size=1, max_size=64),
            process_name=st.text(min_size=1, max_size=64),
            arguments=st.text(min_size=1, max_size=64),
        ),
    )


def get_or_create_process(
    test: unittest.TestCase, graph_client: GraphClient, node_props: ProcessProps
) -> ProcessView:
    node_key = node_key_for_test(test, str(node_props["node_key"]))
    return cast(
        ProcessView, upsert(graph_client, "Process", ProcessView, node_key, node_props)
    )
