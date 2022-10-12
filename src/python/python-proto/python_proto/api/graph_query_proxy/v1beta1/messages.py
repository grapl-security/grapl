from __future__ import annotations

from dataclasses import dataclass

from graplinc.grapl.api.graph_query_proxy.v1beta1 import graph_query_proxy_pb2 as proto
from python_proto.api.graph_query.v1beta1.messages import GraphQuery
from python_proto.common import Uuid
from python_proto.grapl.common.v1beta1.messages import Uid
from python_proto.serde import SerDe


@dataclass(frozen=True, slots=True)
class QueryGraphWithUidRequest(SerDe[proto.QueryGraphWithUidRequest]):
    node_uid: Uid
    graph_query: GraphQuery
    _proto_cls = proto.QueryGraphWithUidRequest

    @classmethod
    def from_proto(
        cls, proto: proto.QueryGraphWithUidRequest
    ) -> QueryGraphWithUidRequest:
        return QueryGraphWithUidRequest(
            tenant_id=Uuid.from_proto(proto.tenant_id),
            node_uid=Uid.from_proto(proto.node_uid),
            graph_query=GraphQuery.from_proto(proto.graph_query),
        )

    def into_proto(self) -> proto.QueryGraphWithUidRequest:
        msg = proto.QueryGraphWithUidRequest()
        msg.graph_query.CopyFrom(self.graph_query.into_proto())
        msg.tenant_id.CopyFrom(self.tenant_id.into_proto())
        msg.node_uid.CopyFrom(self.node_uid.into_proto())
        return msg


@dataclass(frozen=True, slots=True)
class QueryGraphFromUidRequest(SerDe[proto.QueryGraphFromUidRequest]):
    node_uid: Uid
    graph_query: GraphQuery
    _proto_cls = proto.QueryGraphFromUidRequest

    @classmethod
    def from_proto(
        cls, proto: proto.QueryGraphFromUidRequest
    ) -> QueryGraphFromUidRequest:
        return cls(
            tenant_id=Uuid.from_proto(proto.tenant_id),
            node_uid=Uid.from_proto(proto.node_uid),
            graph_query=GraphQuery.from_proto(proto.graph_query),
        )

    def into_proto(self) -> proto.QueryGraphFromUidRequest:
        msg = proto.QueryGraphFromUidRequest()
        msg.graph_query.CopyFrom(self.graph_query.into_proto())
        msg.tenant_id.CopyFrom(self.tenant_id.into_proto())
        msg.node_uid.CopyFrom(self.node_uid.into_proto())
        return msg
