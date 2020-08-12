from collections import defaultdict
from typing import Any, TypeVar, List, Set, Type, Dict, Tuple, Optional, Iterator, Union

from grapl_analyzerlib.node_types import (
    EdgeT,
    PropType,
    PropPrimitive,
    EdgeRelationship,
)
from grapl_analyzerlib.queryable import Queryable, EdgeFilter, ToOneFilter, ToManyFilter
from grapl_analyzerlib.schema import Schema
from grapl_analyzerlib.viewable import Viewable, V, Q
from grapl_analyzerlib.comparators import StrCmp, Eq, Distance


from collections import defaultdict
from typing import Any, TypeVar, List, Set, Type, Dict, Tuple, Optional, Iterator, Union

from grapl_analyzerlib.node_types import (
    EdgeT,
    PropType,
    PropPrimitive,
    EdgeRelationship,
)
from grapl_analyzerlib.queryable import Queryable, EdgeFilter, ToOneFilter, ToManyFilter
from grapl_analyzerlib.schema import Schema
from grapl_analyzerlib.viewable import Viewable, V, Q
from grapl_analyzerlib.comparators import StrCmp, Eq, Distance
from grapl_analyzerlib.nodes.base import BaseView, BaseQuery, BaseSchema

EQ = TypeVar("EQ", bound="EntityQuery")
EV = TypeVar("EV", bound="EntityView")


def default_entity_edges():
    from grapl_analyzerlib.nodes.lens import LensSchema, LensQuery, LensView
    from grapl_analyzerlib.nodes.risk import RiskSchema, RiskQuery, RiskView

    return {
        "in_scope": (
            (EdgeT(EntitySchema, LensSchema, EdgeRelationship.ManyToMany), "scope",)
        ),
        "risks": (
            (
                EdgeT(EntitySchema, RiskSchema, EdgeRelationship.ManyToMany),
                "risky_nodes",
            )
        ),
    }


class EntitySchema(BaseSchema):
    def __init__(self, properties=None, edges=None, view=None):
        from grapl_analyzerlib.nodes.lens import LensSchema

        super(EntitySchema, self).__init__(
            {**(properties or {})},
            {**default_entity_edges(), **(edges or {}),},
            view or EntityView,
        )

    @staticmethod
    def self_type() -> str:
        return "EntityNode"


class EntityQuery(BaseQuery[EV, EQ]):
    def with_lenses(self, *lenses: "LensQuery"):
        lenses = lenses or [LensQuery()]
        self.set_neighbor_filters("in_scope", [lenses])
        for lens in lenses:
            lens.set_neighbor_filters("scope", [self])
        return self

    @staticmethod
    def extend_self(*types):
        raise Exception("It is not possible to extend the EntityQuery")

    @classmethod
    def node_schema(cls) -> "Schema":
        return EntitySchema({}, {}, None)


class EntityView(BaseView[EV, EQ]):
    queryable = EntityQuery

    def __init__(
        self,
        uid: str,
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
        self._kwargs = kwargs

    def get_lenses(self, *lenses, cached=True) -> "List[LensView]":
        if cached and self.lenses:
            return self.lenses

        self_node = (
            self.queryable()
            .with_node_key(eq=self.node_key)
            .with_lenses(*lenses)
            .query_first(self.graph_client)
        )
        if self_node:
            self.lenses = self_node.in_scope
        return self.lenses

    def into_view(self, v: Type["Viewable"]) -> Optional["Viewable"]:
        if v.node_schema().self_type() in self.node_types:
            self.queryable = v.queryable
            return v(
                self.uid,
                self.node_key,
                self.graph_client,
                node_types=self.node_types,
                **self._kwargs,
            )
        return None

    @staticmethod
    def extend_self(*types):
        raise Exception("It is not possible to extend the EntityView")

    @classmethod
    def node_schema(cls) -> "Schema":
        return EntitySchema({}, {}, EntityView)


from grapl_analyzerlib.nodes.lens import LensSchema, LensQuery, LensView
from grapl_analyzerlib.nodes.risk import RiskSchema, RiskQuery, RiskView
