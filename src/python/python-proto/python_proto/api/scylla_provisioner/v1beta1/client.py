from __future__ import annotations

from dataclasses import dataclass

import grpc
from graplinc.grapl.api.scylla_provisioner.v1beta1.scylla_provisioner_pb2_grpc import (
    ScyllaProvisionerServiceStub,
)
from python_proto.api.graph_query.v1beta1.messages import GraphQuery
from python_proto.api.scylla_provisioner.v1beta1.messages import (
    ProvisionGraphForTenantRequest,
    ProvisionGraphForTenantResponse,
)
from python_proto.client import Connectable, GrpcClientConfig
from python_proto.common import Uuid


@dataclass(frozen=True, slots=True)
class ScyllaProvisionerClient(Connectable):
    proto_client: ScyllaProvisionerServiceStub
    client_config: GrpcClientConfig

    # implements Connectable
    @classmethod
    def connect(cls, client_config: GrpcClientConfig) -> ScyllaProvisionerClient:
        address = client_config.address
        channel = grpc.insecure_channel(address)
        stub = ScyllaProvisionerServiceStub(channel)

        return cls(proto_client=stub, client_config=client_config)

    def provision_graph_for_tenant(
        self, tenant_id: Uuid
    ) -> ProvisionGraphForTenantResponse:
        request = ProvisionGraphForTenantRequest(tenant_id=tenant_id)
        proto_response = self.proto_client.ProvisionGraphForTenant(request.into_proto())
        return ProvisionGraphForTenantResponse.from_proto(proto_response)
