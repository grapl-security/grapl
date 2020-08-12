import abc
import types

from copy import deepcopy

from typing import cast, Type, TypeVar, Any, Dict, Tuple, Union


V = TypeVar("V", bound="Viewable")


def default_properties() -> Dict[str, "PropType"]:
    return {
        "uid": PropType(PropPrimitive.Str, False),
        "dgraph.type": PropType(PropPrimitive.Str, True),
    }


class SingletonMeta(type):
    _instances = {}

    def __call__(cls, *args, **kwargs):
        if cls not in cls._instances:
            cls._instances[cls] = super(SingletonMeta, cls).__call__(*args, **kwargs)
        return cls._instances[cls]


class Schema(metaclass=SingletonMeta):
    def __init__(
        self,
        properties: Dict[str, "PropType"],
        edges: Dict[str, Tuple["EdgeT", str]],
        viewable: "Union[Type[Viewable], Callable[[], Type[Viewable]]]",
    ):
        self.properties: Dict[str, "PropType"] = {**default_properties(), **properties}
        self.edges: Dict[str, Tuple["EdgeT", str]] = {}

        for edge_name, (edge, r_edge_name) in edges.items():
            self.add_edge(edge_name, edge, r_edge_name)

        self.viewable = viewable
        self.node_types = {"BaseNode", self.self_type()}

    def add_property(self, prop_name: str, prop: "PropType"):
        self.properties[prop_name] = prop

    def add_edge(self, edge_name: str, edge: "EdgeT", reverse_name: str):
        self.edges[edge_name] = (edge, reverse_name)
        r_edge = edge.reverse()
        self.edges[reverse_name] = (r_edge, edge_name)

    def init_reverse(self):
        for edge_name, (edge, reverse_name) in self.edges.items():
            r_edge = edge.reverse()
            # The edge dest Viewable should already be constructed at this point
            edge.dest().edges[reverse_name] = (r_edge, edge_name)

    def prop_type(self, prop_name: str) -> Union[Tuple["EdgeT", str], "PropType", None]:
        return self.get_properties().get(prop_name) or self.get_edges().get(prop_name)

    def get_edges(self) -> Dict[str, Tuple["EdgeT", str]]:
        return self.edges

    def get_properties(self) -> Dict[str, "PropType"]:
        return self.properties

    def associated_viewable(self) -> Type[V]:
        if isinstance(self.viewable, types.FunctionType):
            self.viewable = self.viewable()

        return cast("Type[V]", self.viewable)

    @staticmethod
    @abc.abstractmethod
    def self_type() -> str:
        raise NotImplementedError
        # noinspection PyUnreachableCode
        return cast(Any, None)  # satisfy pytype

    @staticmethod
    def from_graphdb(type_name: str) -> "Schema":
        """
        Given a `type_name`, queries the graph database
        for the schema of that type, and constructs a Schema
        """
        raise NotImplementedError
        # noinspection PyUnreachableCode
        return cast(Any, None)  # satisfy pytype


from grapl_analyzerlib.node_types import PropType, EdgeT
from grapl_analyzerlib.viewable import Viewable
from grapl_analyzerlib.node_types import PropPrimitive
