from __future__ import annotations
from typing import Any, TypeVar, List, Set, Type, Optional, Callable, Union, Dict, Tuple

from grapl_analyzerlib.node_types import (
    EdgeT,
    EdgeRelationship,
    PropType,
)
from grapl_analyzerlib.nodes.base import BaseView, BaseQuery, BaseSchema
from grapl_analyzerlib.schema import Schema
from grapl_analyzerlib.viewable import Viewable

EQ = TypeVar("EQ", bound="EntityQuery")
EV = TypeVar("EV", bound="EntityView")


def default_entity_edges():
    from grapl_analyzerlib.nodes.lens import LensSchema
    from grapl_analyzerlib.nodes.risk import RiskSchema

    return {
        "in_scope": (
            (
                EdgeT(EntitySchema, LensSchema, EdgeRelationship.ManyToMany),
                "scope",
            )
        ),
        "risks": (
            (
                EdgeT(EntitySchema, RiskSchema, EdgeRelationship.ManyToMany),
                "risky_nodes",
            )
        ),
    }


class EntitySchema(BaseSchema):
    def __init__(
        self,
        properties: "Optional[Dict[str, PropType]]" = None,
        edges: "Optional[Dict[str, Tuple[EdgeT, str]]]" = None,
        view: "Union[Type[Viewable], Callable[[], Type[Viewable]]]" = None,
    ):
        super(EntitySchema, self).__init__(
            properties={**(properties or {})},
            edges={
                **default_entity_edges(),
                **(edges or {}),
            },
            view=(view or EntityView),
        )

    @staticmethod
    def self_type() -> str:
        return "Entity"


class EntityQuery(BaseQuery[EV, EQ]):
    def with_lenses(self, *lenses: "LensQuery"):
        lenses = lenses or [LensQuery()]
        self.set_neighbor_filters("in_scope", [lenses])
        for lens in lenses:
            lens.set_neighbor_filters("scope", [self])
        return self

    def with_risks(self, *risks: "RiskQuery"):
        risks = risks or [RiskQuery()]
        self.set_neighbor_filters("risks", [risks])
        for risk in risks:
            risk.set_neighbor_filters("risky_nodes", [self])
        return self

    @classmethod
    def node_schema(cls) -> Schema:
        return EntitySchema({}, {}, None)


class EntityView(BaseView[EV, EQ]):
    queryable = EntityQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        lenses: "List[LensView]" = None,
        **kwargs,
    ):
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)
        self.node_types = set(node_types)
        self.uid = uid
        self.node_key = node_key
        self.graph_client = graph_client
        self.lenses = lenses or []

    def get_lenses(self, *lenses, cached=False) -> List[LensView]:
        return self.get_neighbor(LensQuery, "in_scope", "scope", lenses, cached) or []

    def get_risks(self, *risks, cached=False) -> List[RiskView]:
        return self.get_neighbor(RiskQuery, "risks", "risky_nodes", risks, cached) or []

    def into_view(self, v: Type[Viewable]) -> Optional[Viewable]:
        if v.node_schema().self_type() in self.node_types:
            self.queryable = v.queryable
            return v(
                self.uid,
                self.node_key,
                self.graph_client,
                node_types=self.node_types,
                **self.predicates,
            )
        return None

    @classmethod
    def node_schema(cls) -> Schema:
        return EntitySchema({}, {}, EntityView)


from grapl_analyzerlib.nodes.lens import LensQuery, LensView
from grapl_analyzerlib.nodes.risk import RiskQuery, RiskView
