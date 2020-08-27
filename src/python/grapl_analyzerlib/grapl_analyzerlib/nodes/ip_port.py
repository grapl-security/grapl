from collections import defaultdict
from typing import Any, TypeVar, List, Set, Type, Dict, Tuple, Optional, Iterator, Union

from grapl_analyzerlib.node_types import (
    EdgeT,
    PropType,
    PropPrimitive,
    EdgeRelationship,
)
from grapl_analyzerlib.queryable import (
    Queryable,
    EdgeFilter,
    ToOneFilter,
    ToManyFilter,
    with_to_neighbor,
    with_str_prop,
    with_int_prop,
)
from grapl_analyzerlib.schema import Schema
from grapl_analyzerlib.viewable import Viewable, V, Q
from grapl_analyzerlib.comparators import StrCmp, Eq, Distance, IntOrNot, StrOrNot
from grapl_analyzerlib.nodes.entity import EntityQuery, EntityView, EntitySchema

IPPQ = TypeVar("IPPQ", bound="IpPortQuery")
IPPV = TypeVar("IPPV", bound="IpPortView")


def default_ip_port_properties():
    return {
        "port": PropType(PropPrimitive.Int, False),
        "ip_address": PropType(PropPrimitive.Str, False),
        "first_seen_timestamp": PropType(PropPrimitive.Int, False),
        "last_seen_timestamp": PropType(PropPrimitive.Int, False),
    }


def default_ip_port_edges() -> Dict[str, Tuple[EdgeT, str]]:
    from grapl_analyzerlib.nodes.network_connection import (
        NetworkConnectionSchema,
        NetworkConnectionQuery,
        NetworkConnectionView,
    )

    return {
        "network_connections": (
            EdgeT(IpPortSchema, NetworkConnectionSchema, EdgeRelationship.ManyToMany),
            "connections_from",
        )
    }


class IpPortSchema(EntitySchema):
    def __init__(self):
        super(IpPortSchema, self).__init__(
            default_ip_port_properties(), default_ip_port_edges(), lambda: IpPortView
        )

    @staticmethod
    def self_type() -> str:
        return "IpPort"


class IpPortQuery(EntityQuery[IPPV, IPPQ]):
    @with_int_prop("port")
    def with_port(
        self,
        *,
        eq: Optional["IntOrNot"] = None,
        gt: Optional["IntOrNot"] = None,
        ge: Optional["IntOrNot"] = None,
        lt: Optional["IntOrNot"] = None,
        le: Optional["IntOrNot"] = None,
    ):
        pass

    @with_str_prop("ip_address")
    def with_ip_address(
        self,
        *,
        eq: Optional["StrOrNot"] = None,
        contains: Optional["OneOrMany[StrOrNot]"] = None,
        starts_with: Optional["StrOrNot"] = None,
        ends_with: Optional["StrOrNot"] = None,
        regexp: Optional["OneOrMany[StrOrNot]"] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ):
        pass

    @with_int_prop("first_seen_timestamp")
    def with_first_seen_timestamp(
        self,
        *,
        eq: Optional["IntOrNot"] = None,
        gt: Optional["IntOrNot"] = None,
        ge: Optional["IntOrNot"] = None,
        lt: Optional["IntOrNot"] = None,
        le: Optional["IntOrNot"] = None,
    ):
        pass

    @with_int_prop("last_seen_timestamp")
    def with_last_seen_timestamp(
        self,
        *,
        eq: Optional["IntOrNot"] = None,
        gt: Optional["IntOrNot"] = None,
        ge: Optional["IntOrNot"] = None,
        lt: Optional["IntOrNot"] = None,
        le: Optional["IntOrNot"] = None,
    ):
        pass

    def with_network_connections(self, *network_connections):
        return self.with_to_neighbor(
            NetworkConnectionQuery,
            "network_connections",
            "connections_from",
            network_connections,
        )

    # @staticmethod
    # def extend_self(*types):
    #     for t in types:
    #         method_list = [
    #             method for method in dir(t) if method.startswith("__") is False
    #         ]
    #         for method in method_list:
    #             setattr(IpPortQuery, method, getattr(t, method))
    #     return type("IpPortQuery", types, {})

    @classmethod
    def node_schema(cls) -> "Schema":
        return IpPortSchema()


class IpPortView(EntityView[IPPV, IPPQ]):
    queryable = IpPortQuery

    def __init__(
        self,
        uid: str,
        node_key: str,
        graph_client,
        node_types: Set[str],
        port: Optional[int] = None,
        ip_address: Optional[str] = None,
        first_seen_timestamp: Optional[int] = None,
        last_seen_timestamp: Optional[int] = None,
        network_connections: Optional[List["NetworkConnectionView"]] = None,
        **kwargs,
    ):
        super(IpPortView, self).__init__(
            uid, node_key, graph_client, node_types, **kwargs
        )
        self.set_predicate("port", port)
        self.set_predicate("ip_address", ip_address)
        self.set_predicate("first_seen_timestamp", first_seen_timestamp)
        self.set_predicate("last_seen_timestamp", last_seen_timestamp)
        self.set_predicate("network_connections", network_connections or [])

    def get_port(self, cached=True):
        return self.get_int("port", cached=cached)

    def get_ip_address(self, cached=True):
        return self.get_str("ip_address", cached=cached)

    def get_first_seen_timestamp(self, cached=True):
        return self.get_int("first_seen_timestamp", cached=cached)

    def get_last_seen_timestamp(self, cached=True):
        return self.get_int("last_seen_timestamp", cached=cached)

    def get_network_connections(self, *network_connections, cached=False):
        return self.get_neighbor(
            NetworkConnectionQuery,
            "network_connections",
            "connections_from",
            network_connections,
            cached=cached,
        )

    @classmethod
    def node_schema(cls) -> "Schema":
        return IpPortSchema()

    # @staticmethod
    # def extend_self(*types):
    #     for t in types:
    #         method_list = [
    #             method for method in dir(t) if method.startswith("__") is False
    #         ]
    #         for method in method_list:
    #             setattr(IpPortView, method, getattr(t, method))
    #     return type("IpPortView", types, {})


from grapl_analyzerlib.nodes.network_connection import (
    NetworkConnectionQuery,
    NetworkConnectionView,
)


class IpPortExtendsNetworkConnectionQuery(NetworkConnectionQuery):
    def with_connections_from(self, *connections_from):
        self.with_to_neighbor(
            IpPortQuery, "connections_from", "network_connections", connections_from
        )


class IpPortExtendsNetworkConnectionView(NetworkConnectionView):
    def get_connections_from(self, *connections_from, cached=False):
        self.get_neighbor(
            IpPortQuery,
            "connections_from",
            "network_connections",
            connections_from,
            cached=cached,
        )


IpPortSchema().init_reverse()

NetworkConnectionQuery = NetworkConnectionQuery.extend_self(
    IpPortExtendsNetworkConnectionQuery
)
NetworkConnectionView = NetworkConnectionView.extend_self(
    IpPortExtendsNetworkConnectionView
)
