from typing import Optional, TypeVar, Tuple, Type, Mapping, Any, Union, cast, List

from pydgraph import DgraphClient

from grapl_analyzerlib.nodes.queryable import Queryable
from grapl_analyzerlib.nodes.viewable import Viewable, EdgeViewT, _ForwardEdgeView, _ReverseEdgeView

# noinspection Mypy

T = TypeVar("T")


class _DynamicNodeQuery(Queryable[T]):
    def __init__(self, node_type: Optional[str], view_type: Type[Viewable[T]]) -> None:
        super(_DynamicNodeQuery, self).__init__(view_type)
        self.node_type = node_type
        self.view_type = view_type
        self.set_str_property_filter('dgraph.type', _str_cmps('dgraph.type', eq=self.node_type))

    def _get_unique_predicate(self) -> 'Optional[Tuple[str, PropertyT]]':
        return None

    def _get_node_type_name(self) -> Optional[str]:
        return self.node_type

    def _get_property_filters(self) -> Mapping[str, 'PropertyFilter[Property]']:
        return self.dynamic_property_filters

    def _get_forward_edges(self) -> Mapping[str, "Queryable[T]"]:
        return self.dynamic_forward_edge_filters

    def _get_reverse_edges(self) -> Mapping[str, Tuple["Queryable[T]", str]]:
        return self.dynamic_reverse_edge_filters

    def query(
            self,
            dgraph_client: DgraphClient,
            contains_node_key: Optional[str] = None,
            first: Optional[int] = 1000,
    ) -> List['Any']:
        return self._query(
            dgraph_client,
            contains_node_key,
            first
        )

    def query_first(
            self, dgraph_client: DgraphClient, contains_node_key: Optional[str] = None
    ) -> Optional['Any']:
        return self._query_first(dgraph_client, contains_node_key)


class _DynamicNodeView(Viewable[T]):
    def __init__(
            self,
            dgraph_client: DgraphClient,
            node_key: str,
            uid: str,
            node_type: str,
            **args: Any,
    ):
        super(_DynamicNodeView, self).__init__(dgraph_client=dgraph_client, node_key=node_key, uid=uid)
        self.node_type = node_type
        if args:
            for arg_name, arg_value in args.items():
                setattr(self, arg_name, arg_value)

    def get_node_type(self) -> str:
        return self.node_type

    @staticmethod
    def _get_property_types() -> Mapping[str, "PropertyT"]:
        return {}

    @staticmethod
    def _get_forward_edge_types() -> Mapping[str, "EdgeViewT"]:
        return {}

    @staticmethod
    def _get_reverse_edge_types() -> Mapping[str, Tuple["EdgeViewT", str]]:
        return {}

    def _get_forward_edges(self) -> 'Mapping[str, _ForwardEdgeView[T]]':
        return {}

    def _get_reverse_edges(self) -> 'Mapping[str,  _ReverseEdgeView[T]]':
        return {}

    def _get_properties(self, fetch: bool = False) -> Mapping[str, Union[str, int]]:
        return cast(Mapping[str, Union[str, int]], {k: v for k, v in self.dynamic_property_types.items()})


DynamicNodeQuery = _DynamicNodeQuery[Any]
DynamicNodeView = _DynamicNodeView[Any]

from grapl_analyzerlib.nodes.comparators import PropertyFilter, _str_cmps
from grapl_analyzerlib.nodes.types import PropertyT, Property
