from __future__ import annotations
from collections import defaultdict
from typing import (
    Any,
    TypeVar,
    List,
    Set,
    Type,
    Dict,
    Tuple,
    Optional,
    Iterator,
    Union,
    TYPE_CHECKING,
)

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
from grapl_analyzerlib.comparators import StrCmp, Eq, Distance
from grapl_analyzerlib.nodes.entity import EntityQuery, EntityView, EntitySchema
from grapl_analyzerlib.comparators import IntOrNot, StrOrNot, OneOrMany


IPQ = TypeVar("IPQ", bound="IpAddressQuery")
IPV = TypeVar("IPV", bound="IpAddressView")


def default_ip_address_properties() -> Dict[str, "PropType"]:
    return {
        "first_seen_timestamp": PropType(PropPrimitive.Int, False),
        "last_seen_timestamp": PropType(PropPrimitive.Int, False),
        "ip_address": PropType(PropPrimitive.Str, False),
    }


def default_ip_address_edges() -> Dict[str, Tuple[EdgeT, str]]:
    from grapl_analyzerlib.nodes.ip_connection import IpConnectionSchema

    return {
        "ip_connections": (
            EdgeT(IpAddressSchema, IpConnectionSchema, EdgeRelationship.ManyToMany),
            "connecting_ips",
        ),
    }


class IpAddressSchema(EntitySchema):
    def __init__(self):
        super(IpAddressSchema, self).__init__(
            default_ip_address_properties(),
            default_ip_address_edges(),
            lambda: IpAddressView,
        )

    @staticmethod
    def self_type() -> str:
        return "IpAddress"


class IpAddressQuery(EntityQuery[IPV, IPQ]):
    @with_int_prop("first_seen_timestamp")
    def with_first_seen_timestamp(
        self,
        *,
        eq: Optional[IntOrNot] = None,
        gt: Optional[IntOrNot] = None,
        ge: Optional[IntOrNot] = None,
        lt: Optional[IntOrNot] = None,
        le: Optional[IntOrNot] = None,
    ):
        pass

    @with_int_prop("last_seen_timestamp")
    def with_last_seen_timestamp(
        self,
        *,
        eq: Optional[IntOrNot] = None,
        gt: Optional[IntOrNot] = None,
        ge: Optional[IntOrNot] = None,
        lt: Optional[IntOrNot] = None,
        le: Optional[IntOrNot] = None,
    ):
        pass

    @with_str_prop("ip_address")
    def with_ip_address(
        self,
        *,
        eq: Optional[StrOrNot] = None,
        contains: Optional[OneOrMany[StrOrNot]] = None,
        starts_with: Optional[StrOrNot] = None,
        ends_with: Optional[StrOrNot] = None,
        regexp: Optional[OneOrMany[StrOrNot]] = None,
        distance_lt: Optional[Tuple[str, int]] = None,
    ) -> ProcessQuery:
        pass

    def with_ip_connections(self, *ip_connections):
        return self.with_to_neighbor(
            IpConnectionQuery(), "ip_connections", "connecting_ips", ip_connections
        )

    @classmethod
    def node_schema(cls) -> "Schema":
        return IpAddressSchema()


class IpAddressView(EntityView[IPV, IPQ]):
    """
    .. list-table::
        :header-rows: 1

        * - Predicate
          - Type
          - Description
        * - node_key
          - string
          - A unique identifier for this node.
        * - ip_address
          - string
          - The IP address that this node represents.
        * - first_seen_timestamp
          - int
          - Time address was first seen (in millis-since-epoch).
        * - last_seen_timestamp
          - int
          - Time address was last seen (in millis-since-epoch).
        * - ip_connections
          - List[:doc:`/nodes/ip_connection`]
          - Connections made from this address.
    """

    queryable = IpAddressQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        first_seen_timestamp: Optional[int] = None,
        last_seen_timestamp: Optional[int] = None,
        ip_address: Optional[str] = None,
        ip_connections: Optional[int] = None,
        **kwargs,
    ):
        super().__init__(uid, node_key, graph_client, node_types, **kwargs)
        self.node_types = set(node_types)

        self.set_predicate("first_seen_timestamp", first_seen_timestamp)
        self.set_predicate("last_seen_timestamp", last_seen_timestamp)
        self.set_predicate("ip_address", ip_address)
        self.set_predicate("ip_connections", ip_connections or [])

    def get_first_seen_timestamp(self, cached=True):
        return self.get_int("first_seen_timestamp", cached=cached)

    def get_last_seen_timestamp(self, cached=True):
        return self.get_int("last_seen_timestamp", cached=cached)

    def get_ip_address(self, cached=True):
        return self.get_str("ip_address", cached=cached)

    def get_ip_connections(self, *ip_connections, cached=False):
        return self.get_neighbor(
            IpConnectionQuery, "ip_connections", "connecting_ips", ip_connections
        )

    @classmethod
    def node_schema(cls) -> "Schema":
        return IpAddressSchema()


from grapl_analyzerlib.nodes.ip_connection import (
    IpConnectionQuery,
    IpConnectionView,
)


class IpAddressExtendsIpConnectionQuery(IpConnectionQuery):
    def with_connecting_ips(self, *connecting_ips: IpAddressSchema):
        return self.with_to_neighbor(
            IpAddressQuery, "connecting_ips", "ip_connection", connecting_ips
        )


class IpAddressExtendsIpConnectionView(IpConnectionView):
    connecting_ips = None

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client: Any,
        node_types: Set[str],
        connecting_ips: Optional[List[IpAddressView]] = None,
        **kwargs,
    ):
        super().__init__(
            uid=uid,
            node_key=node_key,
            graph_client=graph_client,
            node_types=node_types,
            **kwargs,
        )
        self.set_predicate("connecting_ips", connecting_ips or [])

    def get_connecting_ips(self, *connecting_ips: IpAddressSchema, cached=False):
        return self.get_neighbor(
            IpAddressQuery,
            "connecting_ips",
            "ip_connection",
            connecting_ips,
            cached=cached,
        )


IpAddressSchema().init_reverse()

IpConnectionQuery = IpConnectionQuery.extend_self(IpAddressExtendsIpConnectionQuery)
IpConnectionView = IpConnectionView.extend_self(IpAddressExtendsIpConnectionView)

if TYPE_CHECKING:
    from grapl_analyzerlib.nodes.process import ProcessQuery
