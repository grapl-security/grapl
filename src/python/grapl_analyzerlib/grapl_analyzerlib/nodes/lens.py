from typing import Any, TypeVar, List, Set, Dict, Tuple, Optional

from grapl_analyzerlib.node_types import (
    EdgeT,
    PropType,
    PropPrimitive,
    EdgeRelationship,
)
from grapl_analyzerlib.nodes.base import BaseView, BaseQuery, BaseSchema
from grapl_analyzerlib.schema import Schema

LQ = TypeVar("LQ", bound="LensQuery")
LV = TypeVar("LV", bound="LensView")


def default_lens_properties() -> Dict[str, PropType]:
    return {
        "lens_name": PropType(PropPrimitive.Str, False),
        "score": PropType(PropPrimitive.Int, False),
    }


def default_lens_edges() -> Dict[str, Tuple[EdgeT, str]]:
    from grapl_analyzerlib.nodes.entity import EntitySchema

    return {
        "scope": (
            EdgeT(LensSchema, EntitySchema, EdgeRelationship.ManyToMany),
            "in_scope",
        ),
    }


class LensSchema(BaseSchema):
    def __init__(self):
        super(LensSchema, self).__init__(
            default_lens_properties(), default_lens_edges(), lambda: LensView
        )

    @staticmethod
    def self_type() -> str:
        return "Lens"


class LensQuery(BaseQuery[LV, LQ]):

    def with_scope(self, *scope) -> "LensQuery":
        return self.with_to_neighbor(EntityQuery, 'scope', 'in_scope', scope)

    @classmethod
    def node_schema(cls) -> "Schema":
        return LensSchema()

    @staticmethod
    def extend_self(*types):
        for t in types:
            method_list = [
                method for method in dir(t) if method.startswith("__") is False
            ]
            for method in method_list:
                setattr(LensQuery, method, getattr(t, method))
        return type("LensQuery", types, {})


class LensView(BaseView[LV, LQ]):
    queryable = LensQuery

    def __init__(
        self,
        uid: str,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        scope: Optional[List["EntityView"]] = None,
        **kwargs,
    ):
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)

        self.set_predicate('node_types', node_types)
        self.set_predicate('scope', scope or [])

    def get_scope(self, *scope, cached=False):
        return self.get_neighbor(EntityQuery, 'scope', 'in_scope', scope, cached=cached)

    @classmethod
    def node_schema(cls) -> "Schema":
        return LensSchema()

    @staticmethod
    def extend_self(*types):
        for t in types:
            method_list = [
                method for method in dir(t) if method.startswith("__") is False
            ]
            for method in method_list:
                setattr(LensView, method, getattr(t, method))
        return type("LensView", types, {})


from grapl_analyzerlib.nodes.entity import EntityView, EntityQuery
