import pytest

pytest.register_assert_rewrite("python_proto.tests.helpers")

from python_proto.tests.helpers import check_encode_decode_invariant
from python_proto.tests.strategies import (
    decrement_only_int_props,
    decrement_only_uint_props,
    edge_lists,
    edges,
    graph_descriptions,
    id_strategies,
    identified_graphs,
    identified_nodes,
    immutable_int_props,
    immutable_str_props,
    immutable_uint_props,
    increment_only_int_props,
    increment_only_uint_props,
    merged_edge_lists,
    merged_edges,
    merged_graphs,
    merged_nodes,
    node_descriptions,
    node_properties,
    sessions,
    statics,
)


def test_decrement_only_int_prop_encode_decode() -> None:
    check_encode_decode_invariant(decrement_only_int_props())


def test_decrement_only_uint_prop_encode_decode() -> None:
    check_encode_decode_invariant(decrement_only_uint_props())


def test_edge_list_encode_decode() -> None:
    check_encode_decode_invariant(edge_lists())


def test_edge_encode_decode() -> None:
    check_encode_decode_invariant(edges())


def test_graph_description_encode_decode() -> None:
    check_encode_decode_invariant(graph_descriptions())


def test_id_strategy_encode_decode() -> None:
    check_encode_decode_invariant(id_strategies())


def test_identified_graph_encode_decode() -> None:
    check_encode_decode_invariant(identified_graphs())


def test_identified_node_encode_decode() -> None:
    check_encode_decode_invariant(identified_nodes())


def test_immutable_int_prop_encode_decode() -> None:
    check_encode_decode_invariant(immutable_int_props())


def test_immutable_str_prop_encode_decode() -> None:
    check_encode_decode_invariant(immutable_str_props())


def test_immutable_uint_prop_encode_decode() -> None:
    check_encode_decode_invariant(immutable_uint_props())


def test_increment_only_int_prop_encode_decode() -> None:
    check_encode_decode_invariant(increment_only_int_props())


def test_increment_only_uint_prop_encode_decode() -> None:
    check_encode_decode_invariant(increment_only_uint_props())


def test_merged_edge_list_encode_decode() -> None:
    check_encode_decode_invariant(merged_edge_lists())


def test_merged_edge_encode_decode() -> None:
    check_encode_decode_invariant(merged_edges())


def test_merged_graph_encode_decode() -> None:
    check_encode_decode_invariant(merged_graphs())


def test_merged_node_encode_decode() -> None:
    check_encode_decode_invariant(merged_nodes())


def test_node_description_encode_decode() -> None:
    check_encode_decode_invariant(node_descriptions())


def test_node_property_encode_decode() -> None:
    check_encode_decode_invariant(node_properties())


def test_session_encode_decode() -> None:
    check_encode_decode_invariant(sessions())


def test_static_encode_decode() -> None:
    check_encode_decode_invariant(statics())
