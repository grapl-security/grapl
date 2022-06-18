from __future__ import annotations

from collections import defaultdict
from copy import deepcopy

from typing import Dict, Set, Optional, Iterator, Tuple, List, NewType, DefaultDict


class NodeQuery(object):
    def __init__(
            self,
            node_type: str,
            int_filters: Optional[DefaultDict[str, List[List[Cmp]]]] = None,
            str_filters: Optional[DefaultDict[str, List[List[Cmp]]]] = None,
            uid_filters: Optional[DefaultDict[str, IntEq]] = None,
    ) -> None:
        self.node_type = node_type
        self.int_filters = int_filters or defaultdict(list)
        self.str_filters = str_filters or defaultdict(list)
        self.uid_filters = uid_filters or defaultdict(list)
        self.edge_filters = defaultdict(list)

    def with_str_filters(self, property_name: str, filters: List[Cmp]) -> NodeQuery:
        self.str_filters[property_name].append(filters)
        return self

    def with_int_filters(self, property_name: str, filters: List[Cmp]) -> NodeQuery:
        self.int_filters[property_name].append(filters)
        return self

    def with_edge_filters(self, edge_name: str, reverse_edge_name: str, filters: List[NodeQuery]) -> NodeQuery:
        for edge_filter in filters:
            self._add_forward_edge(edge_name, edge_filter)
            edge_filter._add_forward_edge(reverse_edge_name, self)

        return self

    def _add_forward_edge(self, edge_name: str, neighbor: NodeQuery):
        self.edge_filters[edge_name].append(neighbor)


class Visited(object):
    def __init__(self, ) -> None:
        self.visited: Set[(str, str, str)] = set()

    def check(self, source: Queryable, forward_edge_name: str, reverse_edge_name: str, dest: Queryable):
        already_visited = False

        already_visited |= (source._id, forward_edge_name, dest._id) in self.visited
        if not already_visited:
            self.visited.add((source._id, forward_edge_name, dest._id))

        already_visited |= (dest._id, reverse_edge_name, source._id) in self.visited
        if not already_visited:
            self.visited.add((dest._id, reverse_edge_name, source._id))
        return already_visited


def gen_query(
        query: "Queryable",
        edge_mapping: Dict[str, str],
        visited: Visited,
) -> "NodeQuery":
    node_query = NodeQuery(query.node_type)
    for property_name, filters in query._property_filters.items():
        if property_name == "uid":
            node_query.uid_filters[property_name] = filters
        else:
            # TODO: This won't handle `Has` properly
            if isinstance(filters[0][0], StrCmp):
                node_query.str_filters[property_name].append(filters)
            if isinstance(filters[0][0], IntCmp):
                node_query.int_filters[property_name].append(filters)

    for forward_edge_name, edge_filters in query._edge_filters.items():
        reverse_edge_name = query._edge_mapping[forward_edge_name]
        edge_mapping[forward_edge_name] = reverse_edge_name
        edge_mapping[reverse_edge_name] = forward_edge_name

        node_edge_filters = [
            gen_query(edge_filter, edge_mapping, visited) for edge_filter in edge_filters
            if not visited.check(query, forward_edge_name, reverse_edge_name, edge_filter)
        ]

        node_query.with_edge_filters(forward_edge_name, reverse_edge_name, node_edge_filters)

    return node_query


class QueryGraphWithNodeRequest(object):
    def __init__(self, node_uid: int, node_query: NodeQuery, edge_mapping: Dict[str, str]) -> None:
        self.node_uid = node_uid
        self.node_query = node_query
        self.edge_mapping = edge_mapping


class QueryGraphFromNodeRequest(object):
    def __init__(self, node_uid: int, node_query: NodeQuery, edge_mapping: Dict[str, str]) -> None:
        self.node_uid = node_uid
        self.node_query = node_query
        self.edge_mapping = edge_mapping


def gen_query_with_uid(
        root_query: "Queryable",
        uid: int
) -> "QueryGraphWithNodeRequest":
    edge_mapping = {}
    node_query = gen_query(root_query, edge_mapping, Visited())
    return QueryGraphWithNodeRequest(uid, node_query, edge_mapping)


def gen_query_from_uid(
        root_query: "Queryable",
        uid: int
) -> "QueryGraphFromNodeRequest":
    edge_mapping = {}
    node_query = gen_query(root_query, edge_mapping, Visited())
    return QueryGraphFromNodeRequest(uid, node_query, edge_mapping)


def traverse_query_iter(
        root_q: "Queryable", visited: Optional[Set["Queryable"]] = None
) -> Iterator["Queryable"]:
    if visited is None:
        visited = set()

    if root_q in visited:
        return
    yield root_q

    for edge_name, neighbor_filters in root_q.neighbor_filters():
        if not neighbor_filters:
            continue

        visited.add(root_q)

        for neighbor_filter in neighbor_filters:
            if isinstance(neighbor_filter, tuple) or isinstance(neighbor_filter, list):
                for n_filter in neighbor_filter:
                    for nested in traverse_query_iter(n_filter, visited):
                        yield nested
            else:
                for nested in traverse_query_iter(neighbor_filter, visited):
                    yield nested


def traverse_query_neighbors_iter(
        root_q: "Queryable", visited: Optional[Set["Queryable"]] = None
) -> Iterator[Tuple["Queryable", str, "EdgeFilter"]]:
    if visited is None:
        visited = set()

    if root_q in visited:
        return

    for edge_name, neighbor_filters in root_q.neighbor_filters():
        if not neighbor_filters:
            continue
        if root_q not in visited:
            yield root_q, edge_name, neighbor_filters

        visited.add(root_q)

        for neighbor_filter in neighbor_filters:
            if isinstance(neighbor_filter, tuple) or isinstance(neighbor_filter, list):
                for n_filter in neighbor_filter:
                    for nested in traverse_query_neighbors_iter(n_filter, visited):
                        yield nested
            else:
                for nested in traverse_query_neighbors_iter(neighbor_filter, visited):
                    yield nested


from grapl_analyzerlib.queryable import Queryable, EdgeFilter
from grapl_analyzerlib.comparators import (
    Cmp,
    Eq,
    IntEq,
    Has,
    extract_value,
    dgraph_prop_type,
    Contains,
    StartsWith,
    EndsWith,
    Rex,
    StrCmp,
    IntCmp
)
