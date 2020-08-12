import abc
from typing import cast, Any, Dict, Generic, List, Optional, Set, TypeVar, Type, Union, Tuple
from grapl_analyzerlib.extendable import Extendable

MYPY = False

GraphClient = Any  # todo

V = TypeVar("V", bound="Viewable")
Q = TypeVar("Q", bound="Queryable")
T = TypeVar("T")
OneOrMany = Union[List[T], T]

class Viewable(Generic[V, Q], Extendable, abc.ABC):
    queryable = None

    def __init__(
        self, uid: str, node_key: str, graph_client: GraphClient, **kwargs
    ) -> None:
        self.uid = uid
        self.node_key = node_key
        self.graph_client = graph_client
        self.predicates = {}

        for key, value in kwargs.items():
            self.set_predicate(key, value)

    def set_predicate(self, predicate_name: str, predicate: "Union[OneOrMany[str, int, bool], 'Viewable']"):
        self.predicates[predicate_name] = predicate
        setattr(self, predicate_name, predicate)

    def get_str(self, property_name: str, cached=True) -> Optional[str]:
        if cached and getattr(self, property_name, None):
            return getattr(self, property_name, None)

        self_node = (
            self.queryable()
                .with_node_key(eq=self.node_key)
                .with_str_property(property_name)
                .query_first(self.graph_client)
        )

        if self_node:
            self.set_predicate(property_name, getattr(self_node, property_name, None))

        return getattr(self, property_name, None)

    def get_int(self, property_name: str, cached=True) -> Optional[int]:
        if cached and getattr(self, property_name, None):
            return getattr(self, property_name, None)

        self_node = (
            self.queryable()
                .with_node_key(eq=self.node_key)
                .with_int_property(property_name)
                .query_first(self.graph_client)
        )

        if self_node:
            self.set_predicate(property_name, getattr(self_node, property_name, None))

        return getattr(self, property_name, None)

    def get_neighbor(self, default: 'Type[Q]', f_edge: str, r_edge: str, *filters, cached=True) -> Optional['Q']:
        if cached and getattr(self, f_edge, None):
            return getattr(self, f_edge, None)

        self_node = (
            self.queryable()
                .with_node_key(eq=self.node_key)
                .with_to_neighbor(default, f_edge, r_edge, *filters)
                .query_first(self.graph_client)
        )

        if self_node:
            self.set_predicate(f_edge, getattr(self_node, f_edge, None))

        return getattr(self, f_edge, None)

    @classmethod
    def associated_queryable(cls) -> Type[Q]:
        assert cls.queryable, f"{cls.__name__} cls.queryable"
        return cls.queryable

    @classmethod
    @abc.abstractmethod
    def node_schema(cls) -> "Schema":
        raise NotImplementedError
        return cast("Schema", None)

    def get_node_type(self) -> str:
        return self.schema.self_type()

    @classmethod
    def from_dict(cls: Type[V], d: Dict[str, Any], graph_client: Any) -> V:
        self_schema = cls.node_schema()
        self_props = {}

        for name, value in d.items():
            # If it's a property
            ty = self_schema.prop_type(
                name
            )  # type: Union[Tuple[EdgeT, str], PropType, None]
            if ty is None:
                # TODO: Handle this - Construct a placeholder?
                continue
            elif isinstance(ty, PropType):
                deserialized_prop = deserialize_prop(value, ty)
                self_props[name] = deserialized_prop
            elif isinstance(ty[0], EdgeT) and isinstance(ty[1], str):
                edge_ty: EdgeT = ty[0]
                # rev_name: str = ty[
                #     1
                # ]  # TODO: We should add a reverse edge from our neighbor to us

                edge_viewable: Any = edge_ty.dest().associated_viewable()
                self_props[name] = deserialize_edge(
                    edge_viewable, edge_ty, value, graph_client
                )
            else:
                print(name, value, ty)
                raise NotImplementedError
        self_props["node_types"] = self_props.pop("dgraph.type")

        print(self_props)
        self_node = cls(graph_client=graph_client, **self_props)

        return self_node


def deserialize_prop(value, ty: "PropType"):
    if ty.primitive is PropPrimitive.Bool:
        if ty.is_set:
            return set([bool(v) for v in value])
        else:
            return bool(value)

    if ty.primitive is PropPrimitive.Int:
        if ty.is_set:
            return set([int(v) for v in value])
        else:
            return int(value)
    if ty.primitive is PropPrimitive.Str:
        if ty.is_set:
            return set([str(v) for v in value])
        else:
            return str(value)

    raise NotImplementedError


EdgeV = TypeVar("EdgeV", bound="Viewable")


def deserialize_edge(
    edge_viewable: Type[EdgeV], edge_ty: "EdgeT", value, graph_client
) -> Union[EdgeV, List[EdgeV]]:
    if isinstance(value, List):
        edges = []
        assert edge_ty.is_to_many()
        for serialized_edge in value:
            serialized_edge["node_types"] = serialized_edge.pop("dgraph.type")
            edge_view = edge_viewable(graph_client=graph_client, **serialized_edge)
            edges.append(edge_view)
        return edges
    else:
        assert edge_ty.is_to_one(), (edge_ty, value)
        value["node_types"] = value.pop("dgraph.type")
        edge_view = edge_viewable(graph_client=graph_client, **value)
        return edge_view


from grapl_analyzerlib.schema import Schema, EdgeT
from grapl_analyzerlib.node_types import PropType, PropPrimitive
from grapl_analyzerlib.queryable import Queryable
