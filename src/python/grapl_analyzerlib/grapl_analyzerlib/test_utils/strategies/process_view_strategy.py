import unittest
from typing import NewType, Dict, cast
import hypothesis.strategies as st
from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.nodes.process import ProcessView
from grapl_analyzerlib.node_types import PropType
from grapl_analyzerlib.test_utils.dgraph_utils import upsert
from grapl_analyzerlib.test_utils.strategies.misc import (
    build_random_key,
    text_dgraph_compat,
)

ProcessProps = NewType("ProcessProps", Dict[str, PropType])


def process_props() -> st.SearchStrategy[ProcessProps]:
    return st.builds(
        ProcessProps,
        st.builds(
            dict,
            node_key=build_random_key(),
            process_id=st.integers(min_value=1, max_value=2 ** 32),
            created_timestamp=st.integers(min_value=0, max_value=2 ** 48),
            terminate_time=st.integers(min_value=0, max_value=2 ** 48),
            image_name=text_dgraph_compat(),
            process_name=text_dgraph_compat(),
            arguments=text_dgraph_compat(),
        ),
    )


def get_or_create_process(
    test: unittest.TestCase, graph_client: GraphClient, node_props: ProcessProps
) -> ProcessView:
    node_key = node_props["node_key"]
    return cast(
        ProcessView, upsert(graph_client, "Process", ProcessView, node_key, node_props)
    )
