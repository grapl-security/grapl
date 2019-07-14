from collections import defaultdict
from typing import Dict, List, Tuple, Any, Optional

from pydgraph import DgraphClient

from grapl_analyzerlib.node_types import DNQ
from grapl_analyzerlib.querying import PropertyFilter, StrCmp, IntCmp, EdgeFilter, Has, Cmp, _str_cmps, _int_cmps
from grapl_analyzerlib.querying import Viewable
from test import Queryable


class DynamicNodeQuery(Queryable):
    def __init__(self, node_type: str) -> None:
        self.node_type = node_type
        self._node_key = Has(
            "node_key"
        )  # type: Cmp
        self._uid = Has(
            "uid"
        )  # type: Cmp

        # Dict of property name to its associated filters
        self.property_filters = defaultdict(list)  # type: Dict[str, PropertyFilter]

        # Dict of edge name to associated filters
        self.edge_filters = dict()
        self.reverse_edge_filters = dict()

    def with_property_str_filter(self, prop_name: str, eq=StrCmp, contains=StrCmp, ends_with=StrCmp) -> DNQ:
        self.property_filters[prop_name].extend(
            _str_cmps(prop_name, eq, contains, ends_with)
        )
        return self

    def with_property_int_filter(self, prop_name: str, eq=IntCmp, contains=IntCmp, ends_with=IntCmp) -> DNQ:
        self.property_filters[prop_name].extend(
            _int_cmps(prop_name, eq, contains, ends_with)
        )
        return self

    def with_edge_filter(self, edge: str, edge_filter: EdgeFilter) -> DNQ:
        self.edge_filters[edge] = edge_filter
        return self

    def with_reverse_edge_filter(self, edge: str, edge_filter: EdgeFilter) -> DNQ:
        self.reverse_edge_filters[edge] = edge_filter
        return self

    # Querable Interface Implementation
    def get_node_type_name(self) -> str:
        return self.node_type

    def get_node_key_filter(self) -> PropertyFilter:
        return [[self._node_key]]

    def get_uid_filter(self) -> PropertyFilter:
        return [[self._uid]]

    def get_properties(self) -> List[Tuple[str, PropertyFilter]]:
        properties = [(p, f) for (p, f) in self.property_filters.items()]
        properties.append(('node_key', self.get_node_key_filter()))
        properties.append(('uid', self.get_uid_filter()))
        return properties

    def get_forward_edges(self) -> List[Tuple[str, Any]]:
        return [(e, f) for (e, f) in self.edge_filters.items()]

    def get_reverse_edges(self) -> List[Tuple[str, Any]]:
        return [(e, f) for (e, f) in self.reverse_edge_filters.items()]


class DynamicNodeView(Viewable):
    def __init__(
            self,
            dgraph_client: DgraphClient,
            node_key: str,
            node_type: str,
            asset_id: Optional[str] = None,
    ):
        self.dgraph_client = dgraph_client
        self.node_key = node_key
        self.node_type = node_type
        self.asset_id = asset_id
