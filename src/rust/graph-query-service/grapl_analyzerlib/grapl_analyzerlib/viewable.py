from __future__ import annotations
import abc
import logging
import os
import sys
from typing import (
    cast,
    Any,
    Dict,
    Generic,
    List,
    Optional,
    Set,
    TypeVar,
    Type,
    Union,
    Tuple,
    Iterator,
    TYPE_CHECKING, Mapping, DefaultDict,
)

if TYPE_CHECKING:
    from grapl_analyzerlib.queryable import Queryable  # noqa: F401

from grapl_analyzerlib.extendable import Extendable
from grapl_analyzerlib.grapl_client import GraphClient

GRAPL_LOG_LEVEL = os.getenv("GRAPL_LOG_LEVEL")
LEVEL = "ERROR" if GRAPL_LOG_LEVEL is None else GRAPL_LOG_LEVEL
LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(LEVEL)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

V = TypeVar("V", bound="Viewable")
Q = TypeVar("Q", bound="Queryable")
T = TypeVar("T")
OneOrMany = Union[List[T], T]


class Viewable(Generic[V, Q], Extendable, abc.ABC):
    queryable: Type[Q] = None  # pytype: disable=not-supported-yet

    def __init__(
        self, graph_client: GraphClient, node_view: NodeView
    ) -> None:
        self.graph_client = graph_client
        self.node_view = node_view
        self.graph_view = node_view.graph_view

    def get_string(self, property_name: str) -> Optional[str]:
        property_value = self.node_view.string_properties.get(property_name)
        if property_value:
            return property_value

        return self.fetch_string(property_name)

    def fetch_string(self, property_name: str) -> Optional[str]:
        self_node = (
            self._self_query()
                .with_str_property(property_name)
                .query_from_uid(self.graph_client)
        )

        property_value = self_node.node_view.string_properties.get(property_name)
        if property_value:
            self.node_view.string_properties[property_name] = property_value
            return property_value
        return None

    def get_int(self, property_name: str) -> Optional[int]:
        property_value = self.node_view.int_properties.get(property_name)
        if property_value:
            return property_value

        return self.fetch_string(property_name)

    def fetch_int(self, property_name: str) -> Optional[int]:
        self_node = (
            self._self_query()
                .with_int_property(property_name)
                .query_from_uid(self.graph_client)
        )

        property_value = self_node.node_view.int_properties.get(property_name)
        if property_value:
            self.node_view.int_properties[property_name] = property_value
            return property_value
        return None

    def get_neighbors(self, forward_edge_name: str, neighbor_filters: Tuple[Queryable, ...], neighbor_type: Type[Viewable]) -> List[Viewable]:
        self_query = self._self_query()
        self_query.set_neighbor_filters(forward_edge_name, neighbor_filters)
        self_node: Optional[NodeView] = self_query.query_from_uid(self.node_view.uid)
        if not self_node:
            return []

        self.node_view.graph_view.merge(self_node.graph_view)
        edge_views = self.graph_view.edges[forward_edge_name]

        neighbor_uids = [edge_view.dest_uid for edge_view in edge_views]
        neighbor_views = [self.graph_view.node_views[uid] for uid in neighbor_uids]

        return [neighbor_type(self.graph_client, neighbor_view) for neighbor_view in neighbor_views]

    def _self_query(self) -> Q:
        return (
            self.queryable()
                .with_uid(eq=self.graph_client.uid)
                .with_node_type(eq=self.node_view.node_type)
        )


class NodeView(object):
    def __init__(
            self,
            uid: int,
            node_type: str,
            string_properties: Dict[str, str],
            int_properties: Dict[str, int],
            graph_view: GraphView,
    ):
        self.uid = uid
        self.node_type = node_type
        self.string_properties = string_properties
        self.int_properties = int_properties
        self.graph_view = graph_view

    def merge(self, other: "NodeView") -> None:
        self.string_properties.update(other.string_properties)
        self.int_properties.update(other.int_properties)


class EdgeView(object):
    def __init__(self, edge_name: str, source_uid: int, dest_uid: int) -> None:
        self.edge_name = edge_name
        self.source_uid = source_uid
        self.dest_uid = dest_uid


class GraphView(object):
    def __init__(
            self,
            node_views: List[NodeView],
            edges: DefaultDict[str, List[EdgeView]],
    ) -> None:
        self.node_views = node_views
        self.node_views.sort(key=lambda node_view: node_view.uid)
        self.edges = edges

    def merge(self, other: GraphView) -> None:
        for other_node in other.node_views:
            existing_node = self.find_node_by_uid(other_node.uid)
            if existing_node is None:
                self.node_views.append(other_node)
            else:
                existing_node.merge(other_node)

        for edge_name, edge_views in other.edges.items():
            self.edges[edge_name].extend(edge_views)

    def find_node_by_uid(self, uid: int) -> Optional[NodeView]:
        for node in self.node_views:
            if node.uid == uid:
                return node
        return None


from grapl_analyzerlib.schema import Schema, EdgeT
from grapl_analyzerlib.node_types import PropType, PropPrimitive
