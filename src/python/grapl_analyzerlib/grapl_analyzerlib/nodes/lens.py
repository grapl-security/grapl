from __future__ import annotations
import json

from typing import Any, TypeVar, List, Set, Dict, Tuple, Optional

from grapl_analyzerlib.node_types import (
    EdgeT,
    PropType,
    PropPrimitive,
    EdgeRelationship,
)
from grapl_analyzerlib.nodes.base import BaseView, BaseQuery, BaseSchema
from grapl_analyzerlib.schema import Schema
from grapl_analyzerlib.grapl_client import GraphClient

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
        return self.with_to_neighbor(EntityQuery, "scope", "in_scope", scope)

    def with_lens_name(self, eq: str):
        return self.with_str_property("lens_name", eq=eq)

    def with_lens_type(self, eq: str):
        return self.with_str_property("lens_type", eq=eq)

    @classmethod
    def node_schema(cls) -> "Schema":
        return LensSchema()


class LensView(BaseView[LV, LQ]):
    """
    .. list-table::
        :header-rows: 1

        * - Predicate
          - Type
          - Description
        * - node_key
          - string
          - A unique identifier for this node.
        * - lens
          - string
          - The name of the lens this node represents.
        * - scope
          - List[EntityView]
          - todo: documentation
    """

    queryable = LensQuery

    def __init__(
        self,
        uid: str,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        scope: Optional[List["EntityView"]] = None,
        lens_name: Optional[str] = None,
        lens_type: Optional[str] = None,
        **kwargs,
    ):
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)

        self.set_predicate("node_types", node_types)
        self.set_predicate("scope", scope or [])
        self.set_predicate("lens_name", lens_name)
        self.set_predicate("lens_type", lens_type)

    def get_lens_name(self, cached=True):
        return self.get_str("lens_name", cached=cached)

    def get_scope(self, *scope, cached=False):
        return self.get_neighbor(EntityQuery, "scope", "in_scope", scope, cached=cached)

    @staticmethod
    def get_or_create(
        gclient: "GraphClient", lens_name: str, lens_type: str
    ) -> "LensView":
        eg_txn = gclient.txn(read_only=False)
        try:
            query = f"""
            {{
              res(func: eq(node_key, "{'lens-' + lens_type + lens_name}"), first: 1) @cascade
               {{
                 uid,
                 node_type: dgraph.type,
                 node_key,
               }}
             }}"""

            res = eg_txn.query(query)

            res = json.loads(res.json)["res"]
            new_uid = None
            if res:
                new_uid = res[0]["uid"]
            else:
                m_res = eg_txn.mutate(
                    set_obj={
                        "lens_name": lens_name,
                        "lens_type": lens_type,
                        "node_key": "lens-" + lens_type + lens_name,
                        "dgraph.type": "Lens",
                        "score": 0,
                    },
                    commit_now=True,
                )
                uids = m_res.uids

                new_uid = new_uid or uids["blank-0"]
        finally:
            eg_txn.discard()

        self_lens = (
            LensQuery()
            .with_node_key(eq="lens-" + lens_type + lens_name)
            .query_first(gclient)
        )
        assert self_lens, "Lens must exist"
        return self_lens

    @classmethod
    def node_schema(cls) -> "Schema":
        return LensSchema()


from grapl_analyzerlib.nodes.entity import EntityView, EntityQuery

LensSchema().init_reverse()
