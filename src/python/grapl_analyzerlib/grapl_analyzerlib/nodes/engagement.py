from __future__ import annotations

import json
import logging
import os
import sys

from contextlib import contextmanager
from typing import Any, Dict, List, Tuple, Union, Optional, cast, TypeVar

from grapl_analyzerlib.viewable import traverse_view_iter

from grapl_analyzerlib.nodes.entity import EntityView, EntityQuery

from grapl_analyzerlib.grapl_client import GraphClient
from pydgraph import Txn


GRAPL_LOG_LEVEL = os.getenv("GRAPL_LOG_LEVEL")
LEVEL = "ERROR" if GRAPL_LOG_LEVEL is None else GRAPL_LOG_LEVEL
LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(LEVEL)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))


def delete_edge(txn: Txn, from_uid: int, edge_name: str, to_uid: int) -> None:
    if isinstance(from_uid, str):
        from_uid = int(from_uid, 16)

    if isinstance(to_uid, str):
        to_uid = int(to_uid, 16)

    try:
        res = txn.mutate(del_obj=mut, commit_now=True)
        LOGGER.debug("edge mutation result is: {}".format(res))
    finally:
        txn.discard()


def create_edge(txn: Txn, from_uid: int, edge_name: str, to_uid: int) -> None:
    if isinstance(from_uid, str):
        from_uid = int(from_uid, 16)

    if isinstance(to_uid, str):
        to_uid = int(to_uid, 16)

    mut = {"uid": from_uid, edge_name: {"uid": to_uid}}

    try:
        res = txn.mutate(set_obj=mut, commit_now=True)
        LOGGER.debug("edge mutation result is: {}".format(res))
    finally:
        txn.discard()


def stripped_node_to_query(node: Dict[str, Union[str, int]]) -> str:
    func_filter = f'eq(node_key, "{node["node_key"]}")'
    return f"""
        {{
            # stripped_node_to_query
            res(func: {func_filter}, first: 1) {{
                uid,
                node_key,
                dgraph.type: node_type,
            }}
        }}
    """


def get_edges(node: Dict[str, Any]) -> List[Tuple[str, str, str]]:
    edges = []

    for key, value in node.items():
        if isinstance(value, dict):
            edges.append((node["uid"], key, value["uid"]))
        elif isinstance(value, list):
            for neighbor in value:
                if isinstance(neighbor, dict):
                    edges.append((node["uid"], key, neighbor["uid"]))

    return edges


def strip_node(node) -> Dict[str, Any]:
    output = {}
    for key, value in node.items():
        if key == "node_type" or key == "dgraph.type":
            output["dgraph.type"] = value
        if isinstance(value, str) or isinstance(value, int):
            output[key] = value
    return output


def response_into_matrix(res, nodes, edges):
    if isinstance(res, dict):
        edges.extend(get_edges(res))
        nodes[res["node_key"]] = strip_node(res)
        for element in res.values():
            if type(element) is list:
                response_into_matrix(element, nodes, edges)
            if type(element) is dict:
                response_into_matrix(element, nodes, edges)
    else:
        for element in res:
            if type(element) is list:
                response_into_matrix(element, nodes, edges)
            if type(element) is dict:
                response_into_matrix(element, nodes, edges)


class EngagementTransaction(Txn):
    def __init__(
        self, copying_client, eg_uid: int, read_only=False, best_effort=False
    ) -> None:
        super().__init__(copying_client, read_only=read_only, best_effort=best_effort)
        self.eg_uid = eg_uid
        self.client = copying_client.any_client()

    def query(
        self, query, variables=None, timeout=None, metadata=None, credentials=None
    ):
        copied_uids = set()

        txn = self.client.txn(read_only=True)
        try:
            res = txn.query(query, variables, timeout, metadata, credentials)
            decoded_res = json.loads(res.json)
            nodes = {}  # type: Dict[str, Dict[str, Any]]
            edges = []
            response_into_matrix(decoded_res.values(), nodes, edges)
            for node in nodes.values():
                copied_uids.add(node["uid"])
        finally:
            txn.discard()

        for uid in copied_uids:
            if uid == self.eg_uid:
                continue
            create_edge(self.client.txn(read_only=False), self.eg_uid, "scope", uid)
            create_edge(self.client.txn(read_only=False), uid, "in_scope", self.eg_uid)

        return res


class EngagementClient(object):
    def __init__(self, eg_uid: int, gclient: GraphClient):
        self.gclient = gclient
        self.eg_uid = eg_uid

    @staticmethod
    def from_name(engagement_name: str, src_client: GraphClient):

        engagement_lens = LensView.get_or_create(
            src_client, engagement_name, "engagement"
        )
        return EngagementClient(engagement_lens.uid, src_client)

    def any_client(self) -> EngagementClient:
        return self.gclient

    def txn(self, read_only=False, best_effort=False) -> EngagementTransaction:
        return EngagementTransaction(
            self, self.eg_uid, read_only=read_only, best_effort=best_effort
        )

    @contextmanager
    def txn_context(
        self,
        read_only: bool = False,
        best_effort: bool = False,
    ) -> Iterator[Txn]:
        """
        Essentially, this just automates the try-finally in every
        txn() use case, turning it into a context manager.
        It'd be nice to - after a full migration to `txn_context` - perhaps restrict calls to `.txn()`
        """

        txn = self.txn(read_only=False, best_effort=False)
        try:
            yield txn
        finally:
            txn.discard()


from grapl_analyzerlib.nodes.lens import LensView, LensQuery
from grapl_analyzerlib.nodes.base import BaseQuery


EQ = TypeVar("EQ", bound="EngagementQuery")
EV = TypeVar("EV", bound="EngagementView")


class EngagementQuery(BaseQuery[EV, EQ]):
    def with_scope(self, *scope) -> "EngagementQuery":
        return self.with_str_property("lens_type", eq="engagement").with_to_neighbor(
            EntityQuery, "scope", "in_scope", scope
        )

    def with_lens_name(self, eq: str):
        return self.with_str_property("lens_type", eq="engagement").with_str_property(
            "lens", eq=eq
        )


class EngagementView(LensView[EV, EQ]):
    @staticmethod
    def get_or_create(
        eg_client: GraphClient, lens_name: str, lens_type: str = "engagement"
    ) -> "EngagementView":
        lens = LensView.get_or_create(eg_client, lens_name, "engagement")
        engagement_client = EngagementClient(
            lens.uid,
            eg_client,
        )
        lens.graph_client = engagement_client
        return cast("EngagementView", lens.into_view(EngagementView))

    def get_node_by_key(self, node_key: str) -> Optional["EntityView"]:
        return EntityQuery().with_node_key(eq=node_key).query_first(self.graph_client)

    def get_nodes(self, query: EntityQuery, first: int = 100) -> List["EntityView"]:
        return query.query(self.graph_client, first=first)

    def attach_lens_scope(self, lens_name: str, lens_type: str):
        lens_query = LensQuery().with_node_key(eq="lens-" + lens_type + lens_name)
        lens = lens_query.query_first(self.graph_client.gclient)

        if not lens:
            raise Exception(f"Lens with name {lens_name} and type {lens_type} does not exist")

        for node in lens.get_scope():
            self.get_node_by_key(node.node_key)

    def detach(self, *nodes: EntityView, recursive=False):
        for subgraph in nodes:
            if recursive:
                for node in traverse_view_iter(subgraph):
                    remove_from_scope(self, node)
            else:
                remove_from_scope(self, subgraph)


def remove_from_scope(engagement: EngagementView, node: "Viewable"):
    if engagement.node_key == node.node_key:
        return

    txn_0 = engagement.graph_client.gclient.txn(read_only=False)
    delete_edge(txn_0, engagement.uid, "scope", node._get_uid())

    txn_1 = engagement.graph_client.gclient.txn(read_only=False)
    delete_edge(txn_1, node._get_uid(), "in_scope", engagement.uid)


from grapl_analyzerlib.viewable import Viewable
