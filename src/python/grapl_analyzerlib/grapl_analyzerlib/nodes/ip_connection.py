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


IPQ = TypeVar("IPQ", bound="IpConnectionQuery")
IPV = TypeVar("IPV", bound="IpConnectionView")


def default_ip_connection_properties():
    return {
        "src_ip_address": PropType(PropPrimitive.Str, False),
        "src_port": PropType(PropPrimitive.Int, False),
        "dst_ip_address": PropType(PropPrimitive.Str, False),
        "dst_port": PropType(PropPrimitive.Int, False),
        "created_timestamp": PropType(PropPrimitive.Int, False),
        "terminated_timestamp": PropType(PropPrimitive.Int, False),
        "last_seen_timestamp": PropType(PropPrimitive.Int, False),
    }


def default_ip_connection_edges() -> Dict[str, Tuple[EdgeT, str]]:
    return {
        "inbound_ip_connection_to": (
            EdgeT(IpConnectionSchema, IpAddressSchema, EdgeRelationship.ManyToOne),
            "ip_connections_from",
        )
    }


class IpConnectionSchema(EntitySchema):
    def __init__(self):
        super(IpConnectionSchema, self).__init__(
            default_ip_connection_properties(),
            default_ip_connection_edges(),
            lambda: IpConnectionView,
        )

    @staticmethod
    def self_type() -> str:
        return "IpConnection"


class IpConnectionQuery(EntityQuery[IPV, IPQ]):
    @with_str_prop("src_ip_address")
    def with_src_ip_address(
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

    @with_int_prop("src_port")
    def with_src_port(
        self,
        *,
        eq: Optional[IntOrNot] = None,
        gt: Optional[IntOrNot] = None,
        ge: Optional[IntOrNot] = None,
        lt: Optional[IntOrNot] = None,
        le: Optional[IntOrNot] = None,
    ):
        pass

    @with_str_prop("dst_ip_address")
    def with_dst_ip_address(
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

    @with_int_prop("dst_port")
    def with_dst_port(
        self,
        *,
        eq: Optional[IntOrNot] = None,
        gt: Optional[IntOrNot] = None,
        ge: Optional[IntOrNot] = None,
        lt: Optional[IntOrNot] = None,
        le: Optional[IntOrNot] = None,
    ):
        pass

    @with_int_prop("created_timestamp")
    def with_created_timestamp(
        self,
        *,
        eq: Optional[IntOrNot] = None,
        gt: Optional[IntOrNot] = None,
        ge: Optional[IntOrNot] = None,
        lt: Optional[IntOrNot] = None,
        le: Optional[IntOrNot] = None,
    ):
        pass

    @with_int_prop("terminated_timestamp")
    def with_terminated_timestamp(
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

    def with_inbound_ip_connection_to(self, *ip_addresses):
        return self.with_to_neighbor(
            IpConnectionQuery,
            "inbound_ip_connection_to",
            "ip_connections_from",
            ip_addresses,
        )

    @classmethod
    def node_schema(cls) -> "Schema":
        return IpConnectionSchema()


class IpConnectionView(EntityView[IPV, IPQ]):
    """
    .. list-table::
        :header-rows: 1

        * - Predicate
          - Type
          - Description
        * - node_key
          - string
          - A unique identifier for this node.
        * - created_timestamp
          - int
          - Time of the connection creation (in millis-since-epoch).
        * - last_seen_timestamp
          - int
          - Time the connection was last seen (in millis-since-epoch).
        * - terminated_timestamp
          - int
          - Time connection was terminated (in millis-since-epoch).
    """

    queryable = IpConnectionQuery

    def __init__(
        self,
        uid: int,
        node_key: str,
        graph_client,
        node_types: Set[str],
        src_ip_address=None,
        src_port=None,
        dst_ip_address=None,
        dst_port=None,
        created_timestamp=None,
        terminated_timestamp=None,
        last_seen_timestamp=None,
        inbound_ip_connection_to=None,
        **kwargs,
    ):
        super(IpConnectionView, self).__init__(
            uid, node_key, graph_client, node_types, **kwargs
        )

        self.set_predicate("src_ip_address", src_ip_address)
        self.set_predicate("src_port", src_port)
        self.set_predicate("dst_ip_address", dst_ip_address)
        self.set_predicate("dst_port", dst_port)
        self.set_predicate("created_timestamp", created_timestamp)
        self.set_predicate("terminated_timestamp", terminated_timestamp)
        self.set_predicate("last_seen_timestamp", last_seen_timestamp)
        self.set_predicate("inbound_ip_connection_to", inbound_ip_connection_to or [])

    def get_src_ip_address(self, cached=True):
        return self.get_str("src_ip_address", cached=cached)

    def get_src_port(self, cached=True):
        return self.get_int("src_port", cached=cached)

    def get_dst_ip_address(self, cached=True):
        return self.get_str("dst_ip_address", cached=cached)

    def get_dst_port(self, cached=True):
        return self.get_int("dst_port", cached=cached)

    def get_created_timestamp(self, cached=True):
        return self.get_int("created_timestamp", cached=cached)

    def get_terminated_timestamp(self, cached=True):
        return self.get_int("terminated_timestamp", cached=cached)

    def get_last_seen_timestamp(self, cached=True):
        return self.get_int("last_seen_timestamp", cached=cached)

    def get_inbound_ip_connection_to(self, *ip_addresses, cached=False):
        return self.get_neighbor(
            IpConnectionQuery,
            "inbound_ip_connection_to",
            "ip_connections_from",
            ip_addresses,
            cached=cached,
        )

    @classmethod
    def node_schema(cls) -> "Schema":
        return IpConnectionSchema()


from grapl_analyzerlib.nodes.ip_address import (
    IpAddressSchema,
    IpAddressView,
    IpAddressQuery,
)

IpConnectionSchema().init_reverse()


class IpConnectionExtendsIpAddressQuery(IpAddressQuery):
    def with_ip_connections_from(self, *ip_connections_from) -> "IpAddressQuery":
        return self.with_to_neighbor(
            IpConnectionQuery,
            "ip_connections_from",
            "inbound_ip_connection_to",
            ip_connections_from,
        )


class IpConnectionExtendsIpAddressView(IpAddressView):
    def get_ip_connections_from(
        self, *ip_connections_from, cached=False
    ) -> "IpAddressQuery":
        return self.get_neighbor(
            IpConnectionQuery,
            "ip_connections_from",
            "inbound_ip_connection_to",
            ip_connections_from,
            cached=cached,
        )


IpAddressQuery = IpAddressQuery.extend_self(IpConnectionExtendsIpAddressQuery)
IpAddressView = IpAddressView.extend_self(IpConnectionExtendsIpAddressView)

if TYPE_CHECKING:
    from grapl_analyzerlib.nodes.process import ProcessQuery
