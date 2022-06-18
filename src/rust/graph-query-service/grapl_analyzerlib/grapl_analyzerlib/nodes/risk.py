from __future__ import annotations
from typing import Any, TypeVar, List, Dict, Set, Tuple, Optional

from grapl_analyzerlib.node_types import (
    EdgeT,
    PropType,
    PropPrimitive,
    EdgeRelationship,
)
from grapl_analyzerlib.nodes.base import BaseView, BaseQuery, BaseSchema
from grapl_analyzerlib.queryable import with_str_prop, with_int_prop
from grapl_analyzerlib.schema import Schema
from grapl_analyzerlib.comparators import IntOrNot, OneOrMany, StrOrNot

LQ = TypeVar("LQ", bound="RiskQuery")
LV = TypeVar("LV", bound="RiskView")


def default_risk_properties() -> Dict[str, PropType]:
    return {
        "analyzer_name": PropType(PropPrimitive.Str, False),
        "risk_score": PropType(PropPrimitive.Int, False),
    }


def default_risk_edges() -> Dict[str, Tuple[EdgeT, str]]:
    from grapl_analyzerlib.nodes.entity import EntitySchema

    return {
        "risky_nodes": (
            EdgeT(RiskSchema, EntitySchema, EdgeRelationship.ManyToMany),
            "risks",
        )
    }


class RiskSchema(BaseSchema):
    def __init__(self):
        super(RiskSchema, self).__init__(
            default_risk_properties(), default_risk_edges(), lambda: RiskView
        )

    @staticmethod
    def self_type() -> str:
        return "Risk"


class RiskQuery(BaseQuery[LV, LQ]):
    @with_str_prop("analyzer_name")
    def with_analyzer_name(
        self,
        *,
        eq: Optional[StrOrNot] = None,
        contains: Optional[OneOrMany[StrOrNot]] = None,
        starts_with: Optional[StrOrNot] = None,
        ends_with: Optional[StrOrNot] = None,
        regexp: Optional[OneOrMany[StrOrNot]] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        pass

    @with_int_prop("risk_score")
    def with_risk_score(
        self,
        *,
        eq: Optional[IntOrNot] = None,
        gt: Optional[IntOrNot] = None,
        ge: Optional[IntOrNot] = None,
        lt: Optional[IntOrNot] = None,
        le: Optional[IntOrNot] = None,
    ):
        pass

    def with_scope(self, *scope) -> "RiskQuery":
        return self.with_to_neighbor(EntityQuery, "scope", "in_scope", scope)

    @classmethod
    def node_schema(cls) -> "Schema":
        return RiskSchema()


class RiskView(BaseView[LV, LQ]):
    """
    .. list-table::
        :header-rows: 1

        * - Predicate
          - Type
          - Description
        * - node_key
          - string
          - A unique identifier for this node
        * - risk_score
          - int
          - todo: documentation
        * - analyzer_name
          - string
          - The name of the analyzer that spawned this risk.
        * - risky_nodes
          - List[EntityView]
          - todo: documentation
    """

    queryable = RiskQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        risky_nodes: "Optional[List[EntityView]]" = None,
        **kwargs,
    ):
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)
        self.node_types = set(node_types)

        self.set_predicate("risky_nodes", risky_nodes)

    def get_analyzer_name(self, cached=True):
        return self.get_str("analyzer_name", cached=cached)

    def get_risk_score(self, cached=True):
        return self.get_int("risk_score", cached=cached)

    def get_risky_nodes(self, *risks, cached=False) -> "RiskQuery":
        return self.get_neighbor(
            RiskQuery, "risky_nodes", "risks", risks, cached=cached
        )

    @classmethod
    def node_schema(cls) -> "Schema":
        return RiskSchema()


from grapl_analyzerlib.comparators import IntOrNot, StrOrNot
from grapl_analyzerlib.nodes.entity import EntityView, EntityQuery

RiskSchema().init_reverse()
