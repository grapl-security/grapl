import pytest

pytest.register_assert_rewrite("python_proto.tests.helpers")

import hypothesis.strategies as st
from python_proto import common as proto_common_msgs
from python_proto.api.graph_query.v1beta1 import messages as graph_query_msgs
from python_proto.grapl.common.v1beta1 import messages as grapl_common_msgs
from python_proto.tests import strategies
from python_proto.tests.helpers import check_encode_decode_invariant
from python_proto.tests.test_grapl_common import (
    edge_names,
    node_types,
    property_names,
    uids,
)

################################################################################
# Strategies
################################################################################

# Without setting a `max_size` on some of these objects, they become too large
# for data generation.
hypothesis_collections_max_size = 3


def query_ids(
    value: st.SearchStrategy[int] = strategies.uint64s,
) -> st.SearchStrategy[graph_query_msgs.QueryId]:
    return st.builds(graph_query_msgs.QueryId, value=value)


string_operations = st.sampled_from(graph_query_msgs.StringOperation)
int_filter_operations = st.sampled_from(graph_query_msgs.IntFilterOperation)
uid_operations = st.sampled_from(graph_query_msgs.UidOperation)


def string_filters(
    operation: st.SearchStrategy[graph_query_msgs.StringOperation] = string_operations,
    value: st.SearchStrategy[str] = strategies.small_text,
    negated: st.SearchStrategy[bool] = st.booleans(),
) -> st.SearchStrategy[graph_query_msgs.StringFilter]:
    return st.builds(
        graph_query_msgs.StringFilter, operation=operation, value=value, negated=negated
    )


def and_string_filters(
    string_filters: st.SearchStrategy[list[graph_query_msgs.StringFilter]] = st.lists(
        string_filters(),
        max_size=hypothesis_collections_max_size,
    ),
) -> st.SearchStrategy[graph_query_msgs.AndStringFilters]:
    return st.builds(graph_query_msgs.AndStringFilters, string_filters=string_filters)


def or_string_filters(
    and_string_filters: st.SearchStrategy[
        list[graph_query_msgs.AndStringFilters]
    ] = st.lists(
        and_string_filters(),
        max_size=hypothesis_collections_max_size,
    ),
) -> st.SearchStrategy[graph_query_msgs.OrStringFilters]:
    return st.builds(
        graph_query_msgs.OrStringFilters, and_string_filters=and_string_filters
    )


def int_filters(
    operation: st.SearchStrategy[
        graph_query_msgs.IntFilterOperation
    ] = int_filter_operations,
    value: st.SearchStrategy[int] = strategies.int64s,
    negated: st.SearchStrategy[bool] = st.booleans(),
) -> st.SearchStrategy[graph_query_msgs.IntFilter]:
    return st.builds(
        graph_query_msgs.IntFilter, operation=operation, value=value, negated=negated
    )


def and_int_filters(
    int_filters: st.SearchStrategy[list[graph_query_msgs.IntFilter]] = st.lists(
        int_filters(),
        max_size=hypothesis_collections_max_size,
    ),
) -> st.SearchStrategy[graph_query_msgs.AndIntFilters]:
    return st.builds(graph_query_msgs.AndIntFilters, int_filters=int_filters)


def or_int_filters(
    and_int_filters: st.SearchStrategy[list[graph_query_msgs.AndIntFilters]] = st.lists(
        and_int_filters(),
        max_size=hypothesis_collections_max_size,
    ),
) -> st.SearchStrategy[graph_query_msgs.OrIntFilters]:
    return st.builds(graph_query_msgs.OrIntFilters, and_int_filters=and_int_filters)


def edge_query_entries(
    query_id: st.SearchStrategy[graph_query_msgs.QueryId] = query_ids(),
    edge_name: st.SearchStrategy[grapl_common_msgs.EdgeName] = edge_names(),
    neighbor_query_ids: st.SearchStrategy[set[graph_query_msgs.QueryId]] = st.sets(
        query_ids(),
        max_size=hypothesis_collections_max_size,
    ),
) -> st.SearchStrategy[graph_query_msgs.EdgeQueryEntry]:
    return st.builds(
        graph_query_msgs.EdgeQueryEntry,
        query_id=query_id,
        edge_name=edge_name,
        neighbor_query_ids=neighbor_query_ids,
    )


def uid_filters(
    operation: st.SearchStrategy[graph_query_msgs.UidOperation] = uid_operations,
    value: st.SearchStrategy[grapl_common_msgs.Uid] = uids(),
) -> st.SearchStrategy[graph_query_msgs.UidFilter]:
    return st.builds(graph_query_msgs.UidFilter, operation=operation, value=value)


# yeah, we ahve UidFilter and UidFilters
def uid_filters_plural(
    uid_filters: st.SearchStrategy[list[graph_query_msgs.UidFilter]] = st.lists(
        uid_filters(),
        max_size=hypothesis_collections_max_size,
    ),
) -> st.SearchStrategy[graph_query_msgs.UidFilters]:
    return st.builds(graph_query_msgs.UidFilters, uid_filters=uid_filters)


def edge_query_maps(
    entries: st.SearchStrategy[
        dict[graph_query_msgs.EdgeQueryMapK, graph_query_msgs.EdgeQueryMapV]
    ] = st.dictionaries(
        keys=st.tuples(query_ids(), edge_names()),
        values=st.sets(
            query_ids(),
            max_size=hypothesis_collections_max_size,
        ),
        max_size=hypothesis_collections_max_size,
    ),
) -> st.SearchStrategy[graph_query_msgs.EdgeQueryMap]:
    return st.builds(graph_query_msgs.EdgeQueryMap, entries=entries)


def node_property_queries(
    node_type: st.SearchStrategy[grapl_common_msgs.NodeType] = node_types(),
    query_id: st.SearchStrategy[graph_query_msgs.QueryId] = query_ids(),
    string_filters: st.SearchStrategy[
        dict[grapl_common_msgs.PropertyName, graph_query_msgs.OrStringFilters]
    ] = st.dictionaries(
        keys=property_names(),
        values=or_string_filters(),
        max_size=hypothesis_collections_max_size,
    ),
    int_filters: st.SearchStrategy[
        dict[grapl_common_msgs.PropertyName, graph_query_msgs.OrIntFilters]
    ] = st.dictionaries(
        keys=property_names(),
        values=or_int_filters(),
        max_size=hypothesis_collections_max_size,
    ),
    uid_filters: st.SearchStrategy[graph_query_msgs.UidFilters] = uid_filters_plural(),
) -> st.SearchStrategy[graph_query_msgs.NodePropertyQuery]:
    return st.builds(
        graph_query_msgs.NodePropertyQuery,
        node_type=node_type,
        query_id=query_id,
        string_filters=string_filters,
        int_filters=int_filters,
        uid_filters=uid_filters,
    )


def edge_name_maps(
    entries: st.SearchStrategy[
        dict[grapl_common_msgs.EdgeName, grapl_common_msgs.EdgeName]
    ] = st.dictionaries(
        keys=edge_names(),
        values=edge_names(),
        max_size=hypothesis_collections_max_size,
    ),
) -> st.SearchStrategy[graph_query_msgs.EdgeNameMap]:
    return st.builds(graph_query_msgs.EdgeNameMap, entries=entries)


def node_property_query_maps(
    entries: st.SearchStrategy[
        dict[graph_query_msgs.QueryId, graph_query_msgs.NodePropertyQuery]
    ] = st.dictionaries(
        keys=query_ids(),
        values=node_property_queries(),
        max_size=hypothesis_collections_max_size,
    )
) -> st.SearchStrategy[graph_query_msgs.NodePropertyQueryMap]:
    return st.builds(graph_query_msgs.NodePropertyQueryMap, entries=entries)


def graph_queries(
    root_query_id: st.SearchStrategy[graph_query_msgs.QueryId] = query_ids(),
    node_property_queries: st.SearchStrategy[
        graph_query_msgs.NodePropertyQueryMap
    ] = node_property_query_maps(),
    edge_filters: st.SearchStrategy[graph_query_msgs.EdgeQueryMap] = edge_query_maps(),
    edge_map: st.SearchStrategy[graph_query_msgs.EdgeNameMap] = edge_name_maps(),
) -> st.SearchStrategy[graph_query_msgs.GraphQuery]:
    return st.builds(
        graph_query_msgs.GraphQuery,
        root_query_id=root_query_id,
        node_property_queries=node_property_queries,
        edge_filters=edge_filters,
        edge_map=edge_map,
    )


def string_properties(
    properties: st.SearchStrategy[
        dict[grapl_common_msgs.PropertyName, str]
    ] = st.dictionaries(
        keys=property_names(),
        values=strategies.small_text,
        max_size=hypothesis_collections_max_size,
    )
) -> st.SearchStrategy[graph_query_msgs.StringProperties]:
    return st.builds(graph_query_msgs.StringProperties, properties=properties)


def node_properties_views(
    uid: st.SearchStrategy[grapl_common_msgs.Uid] = uids(),
    node_type: st.SearchStrategy[grapl_common_msgs.NodeType] = node_types(),
    string_properties: st.SearchStrategy[
        graph_query_msgs.StringProperties
    ] = string_properties(),
) -> st.SearchStrategy[graph_query_msgs.NodePropertiesView]:
    return st.builds(
        graph_query_msgs.NodePropertiesView,
        uid=uid,
        node_type=node_type,
        string_properties=string_properties,
    )


def node_properties_view_maps(
    entries: st.SearchStrategy[
        dict[grapl_common_msgs.Uid, graph_query_msgs.NodePropertiesView]
    ] = st.dictionaries(
        keys=uids(),
        values=node_properties_views(),
        max_size=hypothesis_collections_max_size,
    )
) -> st.SearchStrategy[graph_query_msgs.NodePropertiesViewMap]:
    return st.builds(graph_query_msgs.NodePropertiesViewMap, entries=entries)


def edge_view_maps(
    entries: st.SearchStrategy[
        dict[graph_query_msgs.EdgeViewMapK, set[grapl_common_msgs.Uid]]
    ] = st.dictionaries(
        keys=st.tuples(
            uids(),
            edge_names(),
        ),
        values=st.sets(
            uids(),
            max_size=hypothesis_collections_max_size,
        ),
        max_size=hypothesis_collections_max_size,
    )
) -> st.SearchStrategy[graph_query_msgs.EdgeViewMap]:
    return st.builds(graph_query_msgs.EdgeViewMap, entries=entries)


def graph_views(
    nodes: st.SearchStrategy[
        graph_query_msgs.NodePropertiesViewMap
    ] = node_properties_view_maps(),
    edges: st.SearchStrategy[graph_query_msgs.EdgeViewMap] = edge_view_maps(),
) -> st.SearchStrategy[graph_query_msgs.GraphView]:
    return st.builds(graph_query_msgs.GraphView, nodes=nodes, edges=edges)


def matched_graph_with_uids(
    matched_graph: st.SearchStrategy[graph_query_msgs.GraphView] = graph_views(),
    root_uid: st.SearchStrategy[grapl_common_msgs.Uid] = uids(),
) -> st.SearchStrategy[graph_query_msgs.MatchedGraphWithUid]:
    return st.builds(
        graph_query_msgs.MatchedGraphWithUid,
        matched_graph=matched_graph,
        root_uid=root_uid,
    )


def no_match_with_uids() -> st.SearchStrategy[graph_query_msgs.NoMatchWithUid]:
    return st.just(graph_query_msgs.NoMatchWithUid())


def maybe_match_with_uids(
    inner: st.SearchStrategy[graph_query_msgs.MaybeMatchWithUidInner] = st.one_of(
        matched_graph_with_uids(),
        no_match_with_uids(),
    )
) -> st.SearchStrategy[graph_query_msgs.MaybeMatchWithUid]:
    return st.builds(graph_query_msgs.MaybeMatchWithUid, inner=inner)


################################################################################
# RPC input/outputs, which should be the ones under test.
################################################################################


def query_graph_with_uid_requests(
    tenant_id: st.SearchStrategy[proto_common_msgs.Uuid] = strategies.uuids(),
    node_uid: st.SearchStrategy[grapl_common_msgs.Uid] = uids(),
    graph_query: st.SearchStrategy[graph_query_msgs.GraphQuery] = graph_queries(),
) -> st.SearchStrategy[graph_query_msgs.QueryGraphWithUidRequest]:
    return st.builds(
        graph_query_msgs.QueryGraphWithUidRequest,
        tenant_id=tenant_id,
        node_uid=node_uid,
        graph_query=graph_query,
    )


def query_graph_from_uid_requests(
    tenant_id: st.SearchStrategy[proto_common_msgs.Uuid] = strategies.uuids(),
    node_uid: st.SearchStrategy[grapl_common_msgs.Uid] = uids(),
    graph_query: st.SearchStrategy[graph_query_msgs.GraphQuery] = graph_queries(),
) -> st.SearchStrategy[graph_query_msgs.QueryGraphFromUidRequest]:
    return st.builds(
        graph_query_msgs.QueryGraphFromUidRequest,
        tenant_id=tenant_id,
        node_uid=node_uid,
        graph_query=graph_query,
    )


def query_graph_from_uid_responses(
    matched_graph: st.SearchStrategy[graph_query_msgs.GraphView] = graph_views(),
) -> st.SearchStrategy[graph_query_msgs.QueryGraphFromUidResponse]:
    return st.builds(
        graph_query_msgs.QueryGraphFromUidResponse, matched_graph=matched_graph
    )


def query_graph_with_uid_responses(
    maybe_match: st.SearchStrategy[
        graph_query_msgs.MaybeMatchWithUid
    ] = maybe_match_with_uids(),
) -> st.SearchStrategy[graph_query_msgs.QueryGraphWithUidResponse]:
    return st.builds(
        graph_query_msgs.QueryGraphWithUidResponse, maybe_match=maybe_match
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
