import json
import logging
import os
import sys

from typing import Any, TypeVar, Set, Type, Optional, List, Dict, Tuple

from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.node_types import (
    EdgeT,
    PropType,
    PropPrimitive,
)
from grapl_analyzerlib.queryable import Queryable
from grapl_analyzerlib.schema import Schema
from grapl_analyzerlib.viewable import Viewable

BQ = TypeVar("BQ", bound="BaseQuery")
BV = TypeVar("BV", bound="BaseView")


GRAPL_LOG_LEVEL = os.getenv("GRAPL_LOG_LEVEL")
LEVEL = "ERROR" if GRAPL_LOG_LEVEL is None else GRAPL_LOG_LEVEL
LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(LEVEL)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))


class BaseSchema(Schema):
    def __init__(
        self,
        properties: "Optional[Dict[str, PropType]]" = None,
        edges: "Optional[Dict[str, Tuple[EdgeT, str]]]" = None,
        view: "Union[Type[Viewable], Callable[[], Type[Viewable]]]" = None,
    ):
        super(BaseSchema, self).__init__(
            {
                **(properties or {}),
                "node_key": PropType(
                    PropPrimitive.Str, False, index=["hash"], upsert=True
                ),
                "last_index_time": PropType(PropPrimitive.Int, False),
            },
            {
                **(edges or {}),
            },
            view or BaseView,
        )

    def generate_type(self) -> str:
        dgraph_builtins = {"dgraph.type", "uid"}

        property_names = [
            p for p in self.properties.keys() if p and p not in dgraph_builtins
        ]
        property_names.extend(self.edges.keys())
        linebreak = "\n" + ("\t" * 4)
        property_str = f"{linebreak}".join(property_names)
        type_str = f"""
            type {self.self_type()} {{
                {property_str}
            }}
        """
        return type_str

    def generate_schema(self) -> str:
        predicates = []
        dgraph_builtins = {"dgraph.type", "uid"}
        for prop_name, prop_type in self.properties.items():
            if prop_name in dgraph_builtins:
                continue
            try:
                prim_str = prop_type.prop_type_str()
                index_str = prop_type.prop_index_str()
                predicates.append(f"{prop_name}: {prim_str} {index_str} .")
            except Exception as e:
                LOGGER.error(f"Failed to generate property schema {prop_name} {e}")
                raise e

        for edge_name, (edge_t, r_name) in self.edges.items():
            if not edge_name:
                continue
            uid_t = "uid"
            if edge_t.is_from_many():
                uid_t = f"[{uid_t}]"
            predicates.append(f"{edge_name}: {uid_t} .")

        return "\n".join(predicates)

    @staticmethod
    def self_type() -> str:
        return "Base"


class BaseQuery(Queryable[BV, BQ]):
    @classmethod
    def node_schema(cls) -> "Schema":
        return BaseSchema()


V = TypeVar("V", bound="Viewable")


class BaseView(Viewable[BV, BQ]):
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

    def into_view(self, v: Type["V"]) -> Optional["V"]:
        if v.node_schema().self_type() in self.node_types:
            self.queryable = v.queryable
            return v(
                uid=self.uid,
                node_key=self.node_key,
                graph_client=self.graph_client,
                node_types=self.node_types,
                **self.predicates,
            )
        return None

    @staticmethod
    def from_node_key(graph_client: GraphClient, node_key: str) -> "Optional[BaseView]":
        self_node = BaseQuery().with_node_key(eq=node_key).query_first(graph_client)

        return self_node

    @staticmethod
    def from_proto(graph_client: GraphClient, node) -> "BaseView":

        from grapl_analyzerlib.prelude import (
            AssetView,
            ProcessView,
            FileView,
            IpConnectionView,
            NetworkConnectionView,
            IpPortView,
            IpAddressView,
            ProcessOutboundConnectionView,
            ProcessInboundConnectionView,
        )

        if node.HasField("process_node"):
            uid = get_uid(graph_client, node.process_node.node_key)
            assert uid

            return ProcessView(
                graph_client=graph_client,
                uid=uid,
                node_key=node.process_node.node_key,
                node_types={"Process"},
            )
        elif node.HasField("file_node"):
            uid = get_uid(graph_client, node.file_node.node_key)

            return FileView(
                graph_client=graph_client,
                uid=uid,
                node_key=node.file_node.node_key,
                node_types={"File"},
            )
        elif node.HasField("asset_node"):
            uid = get_uid(graph_client, node.asset_node.node_key)

            return AssetView(
                uid, node.asset_node.node_key, graph_client, node_types={"Asset"}
            )
        elif node.HasField("ip_address_node"):
            uid = get_uid(graph_client, node.ip_address_node.node_key)

            return IpAddressView(
                uid,
                node.ip_address_node.node_key,
                graph_client,
                node_types={"IpAddress"},
            )
        elif node.HasField("ip_port_node"):
            uid = get_uid(graph_client, node.ip_port_node.node_key)

            return IpPortView(
                uid, node.ip_port_node.node_key, graph_client, node_types={"IpPort"}
            )
        elif node.HasField("process_outbound_connection_node"):
            uid = get_uid(graph_client, node.process_outbound_connection_node.node_key)
            return ProcessOutboundConnectionView(
                uid,
                node.process_outbound_connection_node.node_key,
                graph_client,
                node_types={"ProcessOutboundConnection"},
            )
        elif node.HasField("process_inbound_connection_node"):
            uid = get_uid(graph_client, node.process_inbound_connection_node.node_key)
            return ProcessInboundConnectionView(
                uid,
                node.process_inbound_connection_node.node_key,
                graph_client,
                node_types={"ProcessInboundConnection"},
            )
        elif node.HasField("ip_connection_node"):
            uid = get_uid(graph_client, node.ip_connection_node.node_key)
            return IpConnectionView(
                uid,
                node.ip_connection_node.node_key,
                graph_client,
                node_types={"IpConnection"},
            )
        elif node.HasField("network_connection_node"):
            uid = get_uid(graph_client, node.network_connection_node.node_key)
            return NetworkConnectionView(
                uid,
                node.network_connection_node.node_key,
                graph_client,
                node_types={"NetworkConnection"},
            )

        elif node.HasField("dynamic_node"):
            uid = get_uid(graph_client, node.dynamic_node.node_key)

            return BaseView(
                uid,
                node.dynamic_node.node_key,
                graph_client,
                node_types={node.dynamic_node.node_type},
            )
        else:
            raise Exception(f"Invalid Node Type : {node}")

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
            qres = json.loads(txn.query(query, variables={"$a": self.node_key}).json)
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

    def to_adjacency_list(self):
        from grapl_analyzerlib.viewable import traverse_view_iter
        from collections import defaultdict

        node_dicts = defaultdict(dict)
        edges = defaultdict(list)
        for node in traverse_view_iter(self):
            node_dict = node.to_dict()
            node_dicts[node_dict["node"]["node_key"]] = node_dict["node"]
            edges[node_dict["node"]["node_key"]].extend(node_dict["edges"])

        return {"nodes": node_dicts, "edges": edges}

    def to_dict(self):
        node_dict = {
            "uid": self.uid,
            "node_key": self.node_key,
            "dgraph.type": self.node_schema().self_type(),
        }
        self_key = self.node_key
        edges = []
        for predicate_name, predicate in self.predicates.items():
            if not predicate:
                continue

            if isinstance(predicate, Viewable):
                edges.append(
                    {
                        "from": self_key,
                        "edge_name": predicate_name,
                        "to": predicate.node_key,
                    }
                )
                continue
            elif isinstance(predicate, list) and isinstance(predicate[0], Viewable):
                for p in predicate:
                    edges.append(
                        {
                            "from": self_key,
                            "edge_name": predicate_name,
                            "to": p.node_key,
                        }
                    )
                    continue
            else:
                if isinstance(predicate, set):
                    node_dict[predicate_name] = list(predicate)
                else:
                    if not isinstance(predicate, Viewable) and not (
                        isinstance(predicate, list)
                        and isinstance(predicate[0], Viewable)
                    ):
                        node_dict[predicate_name] = predicate

        return {"node": node_dict, "edges": edges}

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
