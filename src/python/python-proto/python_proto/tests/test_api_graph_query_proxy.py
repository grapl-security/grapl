import pytest

pytest.register_assert_rewrite("python_proto.tests.helpers")

import hypothesis.strategies as st
from python_proto.api.graph_query.v1beta1 import messages as graph_query_msgs
from python_proto.api.graph_query_proxy.v1beta1 import (
    messages as graph_query_proxy_msgs,
)
from python_proto.grapl.common.v1beta1 import messages as grapl_common_msgs
from python_proto.tests import strategies
from python_proto.tests.helpers import check_encode_decode_invariant
from python_proto.tests.test_api_graph_query import (
    graph_queries,
    graph_views,
    maybe_match_with_uids,
)

################################################################################
# RPC input/outputs, which should be the ones under test.
################################################################################


def query_graph_with_uid_requests(
    node_uid: st.SearchStrategy[grapl_common_msgs.Uid] = strategies.uids(),
    graph_query: st.SearchStrategy[graph_query_proxy_msgs.GraphQuery] = graph_queries(),
) -> st.SearchStrategy[graph_query_proxy_msgs.QueryGraphWithUidRequest]:
    return st.builds(
        graph_query_proxy_msgs.QueryGraphWithUidRequest,
        node_uid=node_uid,
        graph_query=graph_query,
    )


def query_graph_with_uid_responses(
    maybe_match: st.SearchStrategy[
        graph_query_msgs.MaybeMatchWithUid
    ] = maybe_match_with_uids(),
) -> st.SearchStrategy[graph_query_msgs.QueryGraphWithUidResponse]:
    return st.builds(
        graph_query_msgs.QueryGraphWithUidResponse, maybe_match=maybe_match
    )


def query_graph_from_uid_requests(
    node_uid: st.SearchStrategy[grapl_common_msgs.Uid] = strategies.uids(),
    graph_query: st.SearchStrategy[graph_query_proxy_msgs.GraphQuery] = graph_queries(),
) -> st.SearchStrategy[graph_query_proxy_msgs.QueryGraphFromUidRequest]:
    return st.builds(
        graph_query_proxy_msgs.QueryGraphFromUidRequest,
        node_uid=node_uid,
        graph_query=graph_query,
    )


def query_graph_from_uid_responses(
    matched_graph: st.SearchStrategy[graph_query_msgs.GraphView] = graph_views(),
) -> st.SearchStrategy[graph_query_msgs.QueryGraphFromUidResponse]:
    return st.builds(
        graph_query_msgs.QueryGraphFromUidResponse, matched_graph=matched_graph
    )


################################################################################
# Tests
################################################################################


def test_query_graph_with_uid_requests() -> None:
    check_encode_decode_invariant(query_graph_with_uid_requests())


def test_query_graph_with_uid_responses() -> None:
    check_encode_decode_invariant(query_graph_with_uid_responses())


def test_query_graph_from_uid_requests() -> None:
    check_encode_decode_invariant(query_graph_from_uid_requests())


def test_query_graph_from_uid_responses() -> None:
    check_encode_decode_invariant(query_graph_from_uid_responses())
