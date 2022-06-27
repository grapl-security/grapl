import uuid

from typing import Optional

from graplinc.grapl.api.graph_query_service.v1beta1.graph_query_service_pb2_grpc import GraphQueryServiceStub
from python_proto.common import Uuid
from python_proto.graplinc.grapl.api.graph_query.v1beta1.messages import QueryGraphWithNodeRequest, \
    QueryGraphWithNodeResponse


class GraphQueryClient(object):
    tenant_id: Uuid
    client_stub: GraphQueryServiceStub

    def __init__(self, tenant_id: Uuid, client_stub: GraphQueryServiceStub) -> None:
        self.tenant_id = tenant_id
        self.client_stub = client_stub

    def query_with_uid(self, request: QueryGraphWithNodeRequest) -> Optional[QueryGraphWithNodeResponse]:
        response = self.client_stub.QueryGraphWithUid(
            request=request.into_proto(),
        )
        return None
