from __future__ import annotations
import abc
import logging
import os
import sys
import types
from typing import cast, Callable, Type, TypeVar, Any, Dict, Tuple, Union

import typing_extensions

IS_LOCAL: typing_extensions.Final[bool] = bool(os.environ.get("IS_LOCAL", False))

GRAPL_LOG_LEVEL = os.getenv("GRAPL_LOG_LEVEL")
LEVEL = "ERROR" if GRAPL_LOG_LEVEL is None else GRAPL_LOG_LEVEL
LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(LEVEL)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

V = TypeVar("V", bound="Viewable")


def default_properties() -> Dict[str, PropType]:
    return {
        "uid": PropType(PropPrimitive.Str, False),
        "dgraph.type": PropType(PropPrimitive.Str, True),
    }


class SingletonMeta(type):
    """
    The SingletonMeta allows is to construct a class only once, globally.
    """

    _instances = {}

    def __call__(cls, *args, **kwargs):
        if cls not in cls._instances:
            cls._instances[cls] = super(SingletonMeta, cls).__call__(*args, **kwargs)
        return cls._instances[cls]


ViewableType = Type["Viewable"]
ReturnsViewableType = Callable[[], ViewableType]


class Schema(metaclass=SingletonMeta):
    """
    Schemas represent an abstract Singleton. Each node type should use a Schema to define itself.

    We use a Singleton pattern in order to allow for arbitrary patching of the schemas. This is necessary
    so that plugin nodes can attach new properties and edges to existing schemas.

    """

    def __init__(
        self,
        properties: Dict[str, PropType],
        edges: Dict[str, Tuple[EdgeT, str]],
        viewable: Union[ViewableType, ReturnsViewableType],
    ):
        self.node_types = {"BaseNode", self.self_type()}
        self.properties: Dict[str, "PropType"] = {**default_properties(), **properties}
        self.edges: Dict[str, Tuple["EdgeT", str]] = {}

        for edge_name, (edge, r_edge_name) in edges.items():
            self.add_edge(edge_name, edge, r_edge_name)

        self.viewable = viewable

    def add_property(self, prop_name: str, prop: PropType):
        self.properties[prop_name] = prop

    def add_edge(self, edge_name: str, edge: EdgeT, reverse_name: str):
        self.edges[edge_name] = (edge, reverse_name)
        if not reverse_name:
            return
        r_edge = edge.reverse()
        self.edges[reverse_name] = (r_edge, edge_name)

    def init_reverse(self):
        for edge_name, (edge, reverse_name) in self.edges.items():
            if not (edge_name and reverse_name):
                continue
            r_edge = edge.reverse()
            # The edge dest Viewable should already be constructed at this point
            edge.dest().edges[reverse_name] = (r_edge, edge_name)

    def prop_type(self, prop_name: str) -> Union[Tuple[EdgeT, str], PropType, None]:
        return self.get_properties().get(prop_name) or self.get_edges().get(prop_name)

    def get_edges(self) -> Dict[str, Tuple[EdgeT, str]]:
        return self.edges

    def get_properties(self) -> Dict[str, PropType]:
        return self.properties

    def associated_viewable(self) -> ViewableType:
        # would be better if self.viewable were Generic
        if isinstance(self.viewable, types.FunctionType):
            self.viewable = cast(ReturnsViewableType, self.viewable)()

        return cast(ViewableType, self.viewable)

    @staticmethod
    @abc.abstractmethod
    def self_type() -> str:
        raise NotImplementedError
        # noinspection PyUnreachableCode
        return cast(Any, None)  # satisfy pytype


from grapl_analyzerlib.node_types import PropType, EdgeT
from grapl_analyzerlib.viewable import Viewable
from grapl_analyzerlib.node_types import PropPrimitive
