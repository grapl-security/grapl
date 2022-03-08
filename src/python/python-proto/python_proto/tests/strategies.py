import datetime
import uuid
from typing import Mapping, Sequence, Union

import hypothesis.strategies as st
from python_proto import SerDe
from python_proto.api import (
    DecrementOnlyIntProp,
    DecrementOnlyUintProp,
    Edge,
    EdgeList,
    GraphDescription,
    IdentifiedGraph,
    IdentifiedNode,
    IdStrategy,
    ImmutableIntProp,
    ImmutableStrProp,
    ImmutableUintProp,
    IncrementOnlyIntProp,
    IncrementOnlyUintProp,
    MergedEdge,
    MergedEdgeList,
    MergedGraph,
    MergedNode,
    NodeDescription,
    NodeProperty,
    Session,
    Static,
)
from python_proto.common import Duration, Timestamp, Uuid
from python_proto.metrics import (
    Counter,
    Gauge,
    GaugeType,
    Histogram,
    Label,
    MetricWrapper,
)
from python_proto.pipeline import Envelope, Metadata, RawLog

#
# constants
#
# These values are used to parametrize the strategies defined below. Some of
# them ensure the generated data actually makes sense. Others are used to
# ensure strategies perform well. Please don't change these without a Very Good
# Reason(TM).


UINT64_MIN = 0
UINT64_MAX = 2 ** 64 - 1

INT64_MIN = -(2 ** 63) + 1
INT64_MAX = 2 ** 63 - 1

INT32_MIN = -(2 ** 31) + 1
INT32_MAX = 2 ** 31 - 1

DURATION_SECONDS_MIN = 0
DURATION_SECONDS_MAX = UINT64_MAX
DURATION_NANOS_MIN = 0
DURATION_NANOS_MAX = 10 ** 9 - 1

MAX_LIST_SIZE = 5

MIN_LOG_EVENT_SIZE = 0
MAX_LOG_EVENT_SIZE = 1024


#
# common
#


def uuids(
    lsbs: st.SearchStrategy[int] = st.integers(
        min_value=UINT64_MIN, max_value=UINT64_MAX
    ),
    msbs: st.SearchStrategy[int] = st.integers(
        min_value=UINT64_MIN, max_value=UINT64_MAX
    ),
) -> st.SearchStrategy[Uuid]:
    return st.builds(Uuid, lsb=lsbs, msb=msbs)


def durations(
    seconds: st.SearchStrategy[int] = st.integers(
        min_value=DURATION_SECONDS_MIN, max_value=DURATION_SECONDS_MAX
    ),
    nanos: st.SearchStrategy[int] = st.integers(
        min_value=DURATION_NANOS_MIN,
        max_value=DURATION_NANOS_MAX,
    ),
) -> st.SearchStrategy[Duration]:
    return st.builds(Duration, seconds=seconds, nanos=nanos)


def timestamps(
    durations: st.SearchStrategy[Duration] = durations(),
    before_epochs: st.SearchStrategy[bool] = st.booleans(),
) -> st.SearchStrategy[Timestamp]:
    return st.builds(Timestamp, duration=durations, before_epoch=before_epochs)


#
# pipeline
#


def metadatas(
    trace_ids: st.SearchStrategy[uuid.UUID] = st.uuids(),
    tenant_ids: st.SearchStrategy[uuid.UUID] = st.uuids(),
    event_source_ids: st.SearchStrategy[uuid.UUID] = st.uuids(),
    created_times: st.SearchStrategy[datetime.datetime] = st.datetimes(),
    last_updated_times: st.SearchStrategy[datetime.datetime] = st.datetimes(),
) -> st.SearchStrategy[Metadata]:
    return st.builds(
        Metadata,
        trace_id=trace_ids,
        tenant_id=tenant_ids,
        event_source_id=event_source_ids,
        created_time=created_times,
        last_updated_time=last_updated_times,
    )


def raw_logs(
    log_events: st.SearchStrategy[bytes] = st.binary(
        min_size=MIN_LOG_EVENT_SIZE, max_size=MAX_LOG_EVENT_SIZE
    )
) -> st.SearchStrategy[RawLog]:
    return st.builds(RawLog, log_event=log_events)


def envelopes(
    metadatas: st.SearchStrategy[Metadata] = metadatas(),
    inner_messages: st.SearchStrategy[SerDe] = uuids()
    | timestamps()
    | durations()
    | raw_logs(),  # TODO: add more here as they're implemented
) -> st.SearchStrategy[Envelope]:
    return st.builds(
        Envelope,
        metadata=metadatas,
        inner_message=inner_messages,
    )


#
# api
#


def sessions(
    primary_key_properties: st.SearchStrategy[Sequence[str]] = st.lists(
        st.text(), max_size=MAX_LIST_SIZE
    ),
    primary_key_requires_asset_ids: st.SearchStrategy[bool] = st.booleans(),
    create_times: st.SearchStrategy[int] = st.integers(
        min_value=UINT64_MIN, max_value=UINT64_MAX
    ),
    last_seen_times: st.SearchStrategy[int] = st.integers(
        min_value=UINT64_MIN, max_value=UINT64_MAX
    ),
    terminate_times: st.SearchStrategy[int] = st.integers(
        min_value=UINT64_MIN, max_value=UINT64_MAX
    ),
) -> st.SearchStrategy[Session]:
    return st.builds(
        Session,
        primary_key_properties=primary_key_properties,
        primary_key_requires_asset_id=primary_key_requires_asset_ids,
        create_time=create_times,
        last_seen_time=last_seen_times,
        terminate_time=terminate_times,
    )


def statics(
    primary_key_properties: st.SearchStrategy[Sequence[str]] = st.lists(
        st.text(), max_size=MAX_LIST_SIZE
    ),
    primary_key_requires_asset_ids: st.SearchStrategy[bool] = st.booleans(),
) -> st.SearchStrategy[Static]:
    return st.builds(
        Static,
        primary_key_properties=primary_key_properties,
        primary_key_requires_asset_id=primary_key_requires_asset_ids,
    )


def id_strategies(
    strategies: st.SearchStrategy[Union[Session, Static]] = st.one_of(
        sessions(), statics()
    )
) -> st.SearchStrategy[IdStrategy]:
    return st.builds(
        IdStrategy,
        strategy=strategies,
    )


def increment_only_uint_props(
    props: st.SearchStrategy[int] = st.integers(
        min_value=UINT64_MIN, max_value=UINT64_MAX
    ),
) -> st.SearchStrategy[IncrementOnlyUintProp]:
    return st.builds(IncrementOnlyUintProp, prop=props)


def immutable_uint_props(
    props: st.SearchStrategy[int] = st.integers(
        min_value=UINT64_MIN, max_value=UINT64_MAX
    ),
) -> st.SearchStrategy[ImmutableUintProp]:
    return st.builds(ImmutableUintProp, prop=props)


def decrement_only_uint_props(
    props: st.SearchStrategy[int] = st.integers(
        min_value=UINT64_MIN, max_value=UINT64_MAX
    ),
) -> st.SearchStrategy[DecrementOnlyUintProp]:
    return st.builds(DecrementOnlyUintProp, prop=props)


def increment_only_int_props(
    props: st.SearchStrategy[int] = st.integers(
        min_value=INT64_MIN, max_value=INT64_MAX
    ),
) -> st.SearchStrategy[IncrementOnlyIntProp]:
    return st.builds(IncrementOnlyIntProp, prop=props)


def immutable_int_props(
    props: st.SearchStrategy[int] = st.integers(
        min_value=INT64_MIN, max_value=INT64_MAX
    ),
) -> st.SearchStrategy[ImmutableIntProp]:
    return st.builds(ImmutableIntProp, prop=props)


def decrement_only_int_props(
    props: st.SearchStrategy[int] = st.integers(
        min_value=INT64_MIN, max_value=INT64_MAX
    ),
) -> st.SearchStrategy[DecrementOnlyIntProp]:
    return st.builds(DecrementOnlyIntProp, prop=props)


def immutable_str_props(
    props: st.SearchStrategy[str] = st.text(),
) -> st.SearchStrategy[ImmutableStrProp]:
    return st.builds(ImmutableStrProp, prop=props)


def node_properties(
    properties: st.SearchStrategy[
        Union[
            IncrementOnlyUintProp,
            DecrementOnlyUintProp,
            ImmutableUintProp,
            IncrementOnlyIntProp,
            DecrementOnlyIntProp,
            ImmutableIntProp,
            ImmutableStrProp,
        ]
    ] = st.one_of(
        increment_only_uint_props(),
        decrement_only_uint_props(),
        immutable_uint_props(),
        increment_only_int_props(),
        decrement_only_int_props(),
        immutable_int_props(),
        immutable_str_props(),
    )
) -> st.SearchStrategy[NodeProperty]:
    return st.builds(NodeProperty, property_=properties)


def node_descriptions(
    properties: st.SearchStrategy[Mapping[str, NodeProperty]] = st.dictionaries(
        keys=st.text(), values=node_properties()
    ),
    node_keys: st.SearchStrategy[str] = st.text(),
    node_types: st.SearchStrategy[str] = st.text(),
    id_strategies: st.SearchStrategy[Sequence[IdStrategy]] = st.lists(
        id_strategies(), max_size=MAX_LIST_SIZE
    ),
) -> st.SearchStrategy[NodeDescription]:
    return st.builds(
        NodeDescription,
        properties=properties,
        node_key=node_keys,
        node_type=node_types,
        id_strategy=id_strategies,
    )


def identified_nodes(
    properties: st.SearchStrategy[Mapping[str, NodeProperty]] = st.dictionaries(
        keys=st.text(), values=node_properties()
    ),
    node_keys: st.SearchStrategy[str] = st.text(),
    node_types: st.SearchStrategy[str] = st.text(),
) -> st.SearchStrategy[IdentifiedNode]:
    return st.builds(
        IdentifiedNode,
        properties=properties,
        node_key=node_keys,
        node_type=node_types,
    )


def merged_nodes(
    properties: st.SearchStrategy[Mapping[str, NodeProperty]] = st.dictionaries(
        keys=st.text(), values=node_properties()
    ),
    uids: st.SearchStrategy[int] = st.integers(
        min_value=UINT64_MIN, max_value=UINT64_MAX
    ),
    node_keys: st.SearchStrategy[str] = st.text(),
    node_types: st.SearchStrategy[str] = st.text(),
) -> st.SearchStrategy[MergedNode]:
    return st.builds(
        MergedNode,
        properties=properties,
        uid=uids,
        node_key=node_keys,
        node_type=node_types,
    )


def edges(
    from_node_keys: st.SearchStrategy[str] = st.text(),
    to_node_keys: st.SearchStrategy[str] = st.text(),
    edge_names: st.SearchStrategy[str] = st.text(),
) -> st.SearchStrategy[Edge]:
    return st.builds(
        Edge,
        from_node_key=from_node_keys,
        to_node_key=to_node_keys,
        edge_name=edge_names,
    )


def edge_lists(
    edges: st.SearchStrategy[Sequence[Edge]] = st.lists(
        edges(), max_size=MAX_LIST_SIZE
    ),
) -> st.SearchStrategy[EdgeList]:
    return st.builds(
        EdgeList,
        edges=edges,
    )


def merged_edges(
    from_uids: st.SearchStrategy[str] = st.text(),
    from_node_keys: st.SearchStrategy[str] = st.text(),
    to_uids: st.SearchStrategy[str] = st.text(),
    to_node_keys: st.SearchStrategy[str] = st.text(),
    edge_names: st.SearchStrategy[str] = st.text(),
) -> st.SearchStrategy[MergedEdge]:
    return st.builds(
        MergedEdge,
        from_uid=from_uids,
        from_node_key=from_node_keys,
        to_uid=to_uids,
        to_node_key=to_node_keys,
        edge_name=edge_names,
    )


def merged_edge_lists(
    edges: st.SearchStrategy[Sequence[MergedEdge]] = st.lists(
        merged_edges(), max_size=MAX_LIST_SIZE
    ),
) -> st.SearchStrategy[MergedEdgeList]:
    return st.builds(
        MergedEdgeList,
        edges=edges,
    )


def graph_descriptions(
    nodes: st.SearchStrategy[Mapping[str, NodeDescription]] = st.dictionaries(
        keys=st.text(), values=node_descriptions()
    ),
    edges: st.SearchStrategy[Mapping[str, EdgeList]] = st.dictionaries(
        keys=st.text(), values=edge_lists()
    ),
) -> st.SearchStrategy[GraphDescription]:
    return st.builds(
        GraphDescription,
        nodes=nodes,
        edges=edges,
    )


def identified_graphs(
    nodes: st.SearchStrategy[Mapping[str, IdentifiedNode]] = st.dictionaries(
        keys=st.text(), values=identified_nodes()
    ),
    edges: st.SearchStrategy[Mapping[str, EdgeList]] = st.dictionaries(
        keys=st.text(), values=edge_lists()
    ),
) -> st.SearchStrategy[IdentifiedGraph]:
    return st.builds(
        IdentifiedGraph,
        nodes=nodes,
        edges=edges,
    )


def merged_graphs(
    nodes: st.SearchStrategy[Mapping[str, MergedNode]] = st.dictionaries(
        keys=st.text(), values=merged_nodes()
    ),
    edges: st.SearchStrategy[Mapping[str, MergedEdgeList]] = st.dictionaries(
        keys=st.text(), values=merged_edge_lists()
    ),
) -> st.SearchStrategy[MergedGraph]:
    return st.builds(
        MergedGraph,
        nodes=nodes,
        edges=edges,
    )


#
# metrics
#


def labels(
    keys: st.SearchStrategy[str] = st.text(),
    values: st.SearchStrategy[str] = st.text(),
) -> st.SearchStrategy[Label]:
    return st.builds(Label, key=keys, value=values)


def counters(
    names: st.SearchStrategy[str] = st.text(),
    increments: st.SearchStrategy[int] = st.integers(
        min_value=UINT64_MIN, max_value=UINT64_MAX
    ),
    labels: st.SearchStrategy[Sequence[Label]] = st.lists(
        labels(), max_size=MAX_LIST_SIZE
    ),
) -> st.SearchStrategy[Counter]:
    return st.builds(Counter, name=names, increment=increments, labels=labels)


def gauge_types() -> st.SearchStrategy[GaugeType]:
    return st.sampled_from(GaugeType)


def gauges(
    gauge_types: st.SearchStrategy[GaugeType] = gauge_types(),
    names: st.SearchStrategy[str] = st.text(),
    values: st.SearchStrategy[float] = st.floats(allow_nan=False, allow_infinity=False),
    labels: st.SearchStrategy[Sequence[Label]] = st.lists(
        labels(), max_size=MAX_LIST_SIZE
    ),
) -> st.SearchStrategy[Gauge]:
    return st.builds(
        Gauge, gauge_type=gauge_types, name=names, value=values, labels=labels
    )


def histograms(
    names: st.SearchStrategy[str] = st.text(),
    values: st.SearchStrategy[float] = st.floats(allow_nan=False, allow_infinity=False),
    labels: st.SearchStrategy[Sequence[Label]] = st.lists(
        labels(), max_size=MAX_LIST_SIZE
    ),
) -> st.SearchStrategy[Histogram]:
    return st.builds(Histogram, name=names, value=values, labels=labels)


def metric_wrappers(
    metrics: st.SearchStrategy[Union[Counter, Gauge, Histogram]] = st.one_of(
        counters(), gauges(), histograms()
    )
) -> st.SearchStrategy[MetricWrapper]:
    return st.builds(MetricWrapper, metric=metrics)
