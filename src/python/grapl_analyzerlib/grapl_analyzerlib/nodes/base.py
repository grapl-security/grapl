import json
from typing import Any, TypeVar, Set, Type, Optional

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
    def from_proto(graph_client: GraphClient, node) -> "BaseView":

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
                graph_client,
                node.ip_address_node.node_key,
                uid,
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


from grapl_analyzerlib.prelude import AssetView, ProcessView, FileView, IpConnectionView, NetworkConnectionView, IpPortView, IpAddressView, ProcessOutboundConnectionView, ProcessInboundConnectionView

