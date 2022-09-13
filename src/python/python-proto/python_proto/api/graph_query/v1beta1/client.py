from __future__ import annotations

from dataclasses import dataclass

from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2_grpc import (
    GraphQueryServiceStub,
)
from python_proto import common as proto_common_msgs
from python_proto.api.graph_query.v1beta1.messages import (
    GraphQuery,
    QueryGraphFromUidRequest,
    QueryGraphFromUidResponse,
    QueryGraphWithUidRequest,
    QueryGraphWithUidResponse,
)
from python_proto.grapl.common.v1beta1.messages import Uid


@dataclass(frozen=True, slots=True)
class GraphQueryClient:
    tenant_id: proto_common_msgs.Uuid
    proto_client: GraphQueryServiceStub

    def query_with_uid(
        self,
        node_uid: Uid,
        graph_query: GraphQuery,
    ) -> QueryGraphWithUidResponse:
        request = QueryGraphWithUidRequest(
            tenant_id=self.tenant_id,
            node_uid=node_uid,
            graph_query=graph_query,
        )
        proto_response = self.proto_client.QueryGraphWithUid(request.into_proto())
        return QueryGraphWithUidResponse.from_proto(proto_response)

    def query_from_uid(
        self,
        node_uid: Uid,
        graph_query: GraphQuery,
    ) -> QueryGraphFromUidResponse:
        request = QueryGraphFromUidRequest(
            tenant_id=self.tenant_id,
            node_uid=node_uid,
            graph_query=graph_query,
        )
        proto_response = self.proto_client.QueryGraphFromUid(request.into_proto())
        return QueryGraphFromUidResponse.from_proto(proto_response)
