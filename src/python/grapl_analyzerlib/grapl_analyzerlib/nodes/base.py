import json
from typing import Any, TypeVar, Set, Type, Optional, List

from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.node_types import (
    PropType,
    PropPrimitive,
)
from grapl_analyzerlib.queryable import Queryable
from grapl_analyzerlib.schema import Schema
from grapl_analyzerlib.viewable import Viewable

BCQ = TypeVar("BCQ", bound="BaseQuery")
BCV = TypeVar("BCV", bound="BaseView")


class BaseSchema(Schema):
    def __init__(self, properties=None, edges=None, view=None):
        super(BaseSchema, self).__init__(
            {**(properties or {}), "node_key": PropType(PropPrimitive.Str, False)},
            {**(edges or {}),},
            view,
        )

    @staticmethod
    def self_type() -> str:
        return "BaseNode"


class BaseQuery(Queryable[BCV, BCQ]):

    # def with_node_key(self, *, eq: str):
    #     self.with_str_property('node_key', eq)

    @staticmethod
    def extend_self(*types):
        for t in types:
            method_list = [
                method for method in dir(t) if method.startswith("__") is False
            ]
            for method in method_list:
                setattr(BaseQuery, method, getattr(t, method))
        return type("BaseQuery", types, {})

    @classmethod
    def node_schema(cls) -> "Schema":
        return BaseSchema()


class BaseView(Viewable[BCV, BCQ]):
    queryable = BaseQuery

    def __init__(
        self,
        uid: str,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        **kwargs,
    ):
        super().__init__(uid, node_key, graph_client, **kwargs)
        self.node_types = node_types
        self.uid = uid
        self.node_key = node_key

    def into_view(self, v: Type["Viewable"]) -> Optional["Viewable"]:
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

    @staticmethod
    def from_node_key(graph_client: GraphClient, node_key: str) -> "Optional[BaseView]":
        self_node = (
            BaseQuery()
            .with_node_key(eq=node_key)
            .query_first(graph_client)
        )

        return self_node

    @staticmethod
    def from_proto(graph_client: GraphClient, node) -> "BaseView":

        from grapl_analyzerlib.prelude import AssetView, ProcessView, FileView, IpConnectionView, NetworkConnectionView, IpPortView, IpAddressView, ProcessOutboundConnectionView, ProcessInboundConnectionView

        if node.HasField("process_node"):
            uid = get_uid(graph_client, node.process_node.node_key)
            assert uid

            return ProcessView(
                graph_client=graph_client,
                uid=uid,
                node_key=node.process_node.node_key,
                node_types={'Process'}
            )
        elif node.HasField("file_node"):
            uid = get_uid(graph_client, node.file_node.node_key)

            return FileView(
                    graph_client=graph_client,
                    uid=uid,
                    node_key=node.file_node.node_key,
                    node_types={'File'}
                )
        elif node.HasField("asset_node"):
            uid = get_uid(graph_client, node.asset_node.node_key)

            return AssetView(
                uid,
                node.asset_node.node_key,
                graph_client,
                {'Asset'}
            )
        elif node.HasField("ip_address_node"):
            uid = get_uid(graph_client, node.ip_address_node.node_key)

            return IpAddressView(
                uid,
                node.ip_address_node.node_key,
                graph_client,
                node_types={'IpAddress'}
            )
        elif node.HasField("ip_port_node"):
            uid = get_uid(graph_client, node.ip_port_node.node_key)

            return IpPortView(
                uid,
                node.ip_port_node.node_key,
                graph_client,
                {'IpPort'}
            )
        elif node.HasField("process_outbound_connection_node"):
            uid = get_uid(graph_client, node.process_outbound_connection_node.node_key)
            return ProcessOutboundConnectionView(
                uid,
                node.process_outbound_connection_node.node_key,
                graph_client,
                {'ProcessOutboundConnection'}
            )
        elif node.HasField("process_inbound_connection_node"):
            uid = get_uid(graph_client, node.process_inbound_connection_node.node_key)
            return ProcessInboundConnectionView(
                    uid,
                    node.process_inbound_connection_node.node_key,
                    graph_client,
                    {'ProcessInboundConnection'}
                )
        elif node.HasField("ip_connection_node"):
            uid = get_uid(graph_client, node.ip_connection_node.node_key)
            return IpConnectionView(
                uid,
                node.ip_connection_node.node_key,
                graph_client,
                {'IpConnection'}
            )
        elif node.HasField("network_connection_node"):
            uid = get_uid(graph_client, node.network_connection_node.node_key)
            return NetworkConnectionView(
                    uid,
                    node.network_connection_node.node_key,
                    graph_client,
                    {'NetworkConnection'}
                )

        elif node.HasField("dynamic_node"):
            uid = get_uid(graph_client, node.dynamic_node.node_key)

            return BaseView(
                uid,
                    node.dynamic_node.node_key,
                graph_client,
                    {node.dynamic_node.node_type}
                )
        else:
            raise Exception(f"Invalid Node Type : {node}")

    @staticmethod
    def extend_self(*types):
        for t in types:
            method_list = [
                method for method in dir(t) if method.startswith("__") is False
            ]
            for method in method_list:
                setattr(BaseView, method, getattr(t, method))
        return type("BaseView", types, {})

    @classmethod
    def node_schema(cls) -> "Schema":
        return BaseSchema({}, {}, BaseView)

    def _expand(self, edge_str: Optional[List[str]] = None):
        # get the raw dictionary for this type
        if edge_str:
            edge_filters = " AND " + " AND ".join(edge_str or [])
        else:
            edge_filters = ""
        query = f"""
        query q0($a: string) {{
            edges(func: eq(node_key, $a) , first: 1) {{
                uid
                dgraph.type
                node_key
                expand(_all_) @filter(has(dgraph.type) AND has(node_key) {edge_filters}) {{
                    uid
                    dgraph.type
                    expand(_all_)
                }}
            }}

            properties(func: eq(node_key, $a) , first: 1) {{
                uid
                dgraph.type
                expand(_all_)
            }}
        }}
        """
        txn = self.graph_client.txn(read_only=True, best_effort=True)

        try:
            qres = json.loads(txn.query(query, variables={'$a': self.node_key}).json)
        finally:
            txn.discard()

        d = qres.get("edges")
        if d:
            self_node = BaseView.from_dict(d[0], self.graph_client)
            self.predicates = {**self.predicates, **self_node.predicates}

        d = qres.get("properties")
        if d:
            self_node = BaseView.from_dict(d[0], self.graph_client)
            self.predicates = {**self.predicates, **self_node.predicates}

        return None


    # def expand_neighbors(self, filter):
    #     # get the raw dictionary for this type
    #     query = f"""
    #         query res($a: string)
    #         {{
    #             query(func: uid($a, first: 1) {{
    #               expand(_all_)
    #             }}
    #         }}
    #     """
    #     txn = self.graph_client.txn(read_only=True, best_effort=True)
    #
    #     try:
    #         res = txn.query(query, variables={"$a": self.uid})
    #         res = json.loads(res.json)['query']
    #         if not res:
    #             return
    #
    #         if isinstance(res, list):
    #             self_node = BaseView.from_dict(res[0], self.graph_client)
    #         else:
    #             self_node = BaseView.from_dict(res, self.graph_client)
    #         self.predicates = {**self_node.predicates, **self.predicates}
    #     finally:
    #         txn.discard()


# Proto nodes don't contain a uid so we have to fetch them. It may make sense to store these uids
# alongside the proto in the future. This makes constructing from proto relatively expensive.
def get_uid(client: GraphClient, node_key: str) -> str:
    txn = client.txn(read_only=True)
    try:
        query = """
            query res($a: string)
            {
              res(func: eq(node_key, $a), first: 1) @cascade
               {
                 uid,
               }
             }"""
        res = txn.query(query, variables={"$a": node_key})
        res = json.loads(res.json)

        if isinstance(res["res"], list):
            if res["res"]:
                return str(res["res"][0]["uid"])
            else:
                raise Exception(f"get_uid failed for node_key: {node_key} {res}")
        else:
            return str(res["res"]["uid"])

    finally:
        txn.discard()

