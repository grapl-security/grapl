import pytest

pytest.register_assert_rewrite("python_proto.tests.helpers")

import hypothesis.strategies as st
from python_proto import common as proto_common_msgs
from python_proto.api.plugin_sdk.analyzers.v1beta1 import messages as analyzer_msgs
from python_proto.grapl.common.v1beta1 import messages as grapl_common_msgs
from python_proto.tests import strategies
from python_proto.tests.helpers import check_encode_decode_invariant
from python_proto.tests.test_grapl_common import edge_names, property_names, uids

################################################################################
# Strategies
################################################################################


def string_property_updates(
    uid: st.SearchStrategy[grapl_common_msgs.Uid] = uids(),
    property_name: st.SearchStrategy[grapl_common_msgs.PropertyName] = property_names(),
) -> st.SearchStrategy[analyzer_msgs.StringPropertyUpdate]:
    return st.builds(
        analyzer_msgs.StringPropertyUpdate, uid=uid, property_name=property_name
    )


def uint64_property_updates(
    uid: st.SearchStrategy[grapl_common_msgs.Uid] = uids(),
    property_name: st.SearchStrategy[grapl_common_msgs.PropertyName] = property_names(),
) -> st.SearchStrategy[analyzer_msgs.UInt64PropertyUpdate]:
    return st.builds(
        analyzer_msgs.UInt64PropertyUpdate, uid=uid, property_name=property_name
    )


def int64_property_updates(
    uid: st.SearchStrategy[grapl_common_msgs.Uid] = uids(),
    property_name: st.SearchStrategy[grapl_common_msgs.PropertyName] = property_names(),
) -> st.SearchStrategy[analyzer_msgs.Int64PropertyUpdate]:
    return st.builds(
        analyzer_msgs.Int64PropertyUpdate, uid=uid, property_name=property_name
    )


def edge_updates(
    src_uid: st.SearchStrategy[grapl_common_msgs.Uid] = uids(),
    dst_uid: st.SearchStrategy[grapl_common_msgs.Uid] = uids(),
    forward_edge_name: st.SearchStrategy[grapl_common_msgs.EdgeName] = edge_names(),
    reverse_edge_name: st.SearchStrategy[grapl_common_msgs.EdgeName] = edge_names(),
) -> st.SearchStrategy[analyzer_msgs.EdgeUpdate]:
    return st.builds(
        analyzer_msgs.EdgeUpdate,
        src_uid=src_uid,
        dst_uid=dst_uid,
        forward_edge_name=forward_edge_name,
        reverse_edge_name=reverse_edge_name,
    )


def updates(
    inner: st.SearchStrategy[analyzer_msgs.UpdateInner] = st.one_of(
        string_property_updates(),
        uint64_property_updates(),
        int64_property_updates(),
        edge_updates(),
    )
) -> st.SearchStrategy[analyzer_msgs.Update]:
    return st.builds(analyzer_msgs.Update, inner=inner)


def run_analyzer_requests(
    tenant_id: st.SearchStrategy[proto_common_msgs.Uuid] = strategies.uuids(),
    update: st.SearchStrategy[analyzer_msgs.Update] = updates(),
) -> st.SearchStrategy[analyzer_msgs.RunAnalyzerRequest]:
    return st.builds(
        analyzer_msgs.RunAnalyzerRequest, tenant_id=tenant_id, update=update
    )


def analyzer_names(
    value: st.SearchStrategy[str] = st.text(),
) -> st.SearchStrategy[analyzer_msgs.AnalyzerName]:
    return st.builds(analyzer_msgs.AnalyzerName, value=value)


def lens_refs(
    lens_namespace: st.SearchStrategy[str] = st.text(),
    lens_name: st.SearchStrategy[str] = st.text(),
) -> st.SearchStrategy[analyzer_msgs.LensRef]:
    return st.builds(
        analyzer_msgs.LensRef, lens_namespace=lens_namespace, lens_name=lens_name
    )


def execution_hits(
    lens_refs: st.SearchStrategy[list[analyzer_msgs.LensRef]] = st.lists(lens_refs()),
    analyzer_name: st.SearchStrategy[analyzer_msgs.AnalyzerName] = analyzer_names(),
    time_of_match: st.SearchStrategy[
        proto_common_msgs.Timestamp
    ] = strategies.timestamps(),
    idempotency_key: st.SearchStrategy[int] = strategies.uint64s,
    score: st.SearchStrategy[int] = strategies.int32s,
) -> st.SearchStrategy[analyzer_msgs.ExecutionHit]:
    return st.builds(
        analyzer_msgs.ExecutionHit,
        lens_refs=lens_refs,
        analyzer_name=analyzer_name,
        time_of_match=time_of_match,
        idempotency_key=idempotency_key,
        score=score,
    )


def execution_misses() -> st.SearchStrategy[analyzer_msgs.ExecutionMiss]:
    return st.builds(analyzer_msgs.ExecutionMiss)


def execution_results(
    inner: st.SearchStrategy[analyzer_msgs.ExecutionResultInner] = st.one_of(
        execution_hits(),
        execution_misses(),
    )
) -> st.SearchStrategy[analyzer_msgs.ExecutionResult]:
    return st.builds(analyzer_msgs.ExecutionResult, inner=inner)


def run_analyzer_responses(
    execution_result: st.SearchStrategy[
        analyzer_msgs.ExecutionResult
    ] = execution_results(),
) -> st.SearchStrategy[analyzer_msgs.RunAnalyzerResponse]:
    return st.builds(
        analyzer_msgs.RunAnalyzerResponse, execution_result=execution_result
    )


################################################################################
# Tests
################################################################################


def test_run_analyzer_requests() -> None:
    check_encode_decode_invariant(run_analyzer_requests())


def test_run_analyzer_responses() -> None:
    check_encode_decode_invariant(run_analyzer_responses())
