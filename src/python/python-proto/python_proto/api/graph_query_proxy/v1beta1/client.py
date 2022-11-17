from __future__ import annotations

from dataclasses import dataclass

import grpc
from grapl_common.logger import get_structlogger
from grapl_common.retry import retry
from graplinc.grapl.api.graph_query_proxy.v1beta1.graph_query_proxy_pb2_grpc import (
    GraphQueryProxyServiceStub,
)
from python_proto.api.graph_query.v1beta1.messages import GraphQuery
from python_proto.api.graph_query_proxy.v1beta1.messages import (
    QueryGraphFromUidRequest,
    QueryGraphFromUidResponse,
    QueryGraphWithUidRequest,
    QueryGraphWithUidResponse,
)
from python_proto.client import Connectable, GrpcClientConfig
from python_proto.grapl.common.v1beta1.messages import Uid
from python_proto.metadata import GrpcOutboundMetadata, metadata_to_raw

LOGGER = get_structlogger()


@dataclass(frozen=True, slots=True)
class GraphQueryProxyClient(Connectable):
    proto_client: GraphQueryProxyServiceStub
    client_config: GrpcClientConfig

    # implements Connectable
    @classmethod
    def connect(cls, client_config: GrpcClientConfig) -> GraphQueryProxyClient:
        address = client_config.address
        channel = grpc.insecure_channel(address)
        stub = GraphQueryProxyServiceStub(channel)

        return cls(proto_client=stub, client_config=client_config)

    @retry(
        Exception,
        logger=LOGGER,
    )
    def query_with_uid(
        self,
        node_uid: Uid,
        graph_query: GraphQuery,
        metadata: GrpcOutboundMetadata | None = None,
    ) -> QueryGraphWithUidResponse:
        request = QueryGraphWithUidRequest(
            node_uid=node_uid,
            graph_query=graph_query,
        )
        proto_response = self.proto_client.QueryGraphWithUid(
            request.into_proto(),
            timeout=5,
            metadata=metadata_to_raw(metadata),
        )
        return QueryGraphWithUidResponse.from_proto(proto_response)

    @retry(
        Exception,
        logger=LOGGER,
    )
    def query_from_uid(
        self,
        node_uid: Uid,
        graph_query: GraphQuery,
        metadata: GrpcOutboundMetadata | None = None,
    ) -> QueryGraphFromUidResponse:
        request = QueryGraphFromUidRequest(
            node_uid=node_uid,
            graph_query=graph_query,
        )
        proto_response = self.proto_client.QueryGraphFromUid(
            request.into_proto(),
            timeout=5,
            metadata=metadata_to_raw(metadata),
        )
        return QueryGraphFromUidResponse.from_proto(proto_response)
