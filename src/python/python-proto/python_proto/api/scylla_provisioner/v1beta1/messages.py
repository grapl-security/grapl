from __future__ import annotations

from dataclasses import dataclass

from graplinc.grapl.api.scylla_provisioner.v1beta1 import (
    scylla_provisioner_pb2 as proto,
)
from python_proto.common import Uuid
from python_proto.serde import SerDe


@dataclass(frozen=True, slots=True)
class ProvisionGraphForTenantRequest(SerDe[proto.ProvisionGraphForTenantRequest]):
    tenant_id: Uuid

    _proto_cls = proto.ProvisionGraphForTenantResponse

    @classmethod
    def from_proto(
        cls, proto: proto.ProvisionGraphForTenantRequest
    ) -> ProvisionGraphForTenantRequest:
        return cls(tenant_id=Uuid.from_proto(proto.tenant_id))

    def into_proto(self) -> proto.ProvisionGraphForTenantRequest:
        msg = self.new_proto()
        msg.tenant_id.CopyFrom(self.tenant_id.into_proto())
        return msg


@dataclass(frozen=True, slots=True)
class ProvisionGraphForTenantResponse(SerDe[proto.ProvisionGraphForTenantResponse]):
    _proto_cls = proto.ProvisionGraphForTenantResponse

    @classmethod
    def from_proto(
        cls,
        proto: proto.ProvisionGraphForTenantResponse,
    ) -> ProvisionGraphForTenantResponse:
        return cls()

    def into_proto(self) -> proto.ProvisionGraphForTenantResponse:
        msg = self.new_proto()
        return msg
