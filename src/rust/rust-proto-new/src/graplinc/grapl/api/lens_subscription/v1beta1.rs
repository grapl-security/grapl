#![allow(warnings)]

use crate::{
    protobufs::graplinc::grapl::api::lens_subscription::v1beta1::{
        operation::GraphOperations as GraphOperationsProto,
        CreateEdge as CreateEdgeProto,
        CreateNode as CreateNodeProto,
        DeleteEdge as DeleteEdgeProto,
        DeleteNode as DeleteNodeProto,
        LensSubscription as LensSubscriptionProto,
        LensUpdate as LensUpdateProto,
        Operation as OperationProto,
        SubscribeToLensRequest as SubscribeToLensRequestProto,
        SubscribeToLensResponse as SubscribeToLensResponseProto,
    },
    serde_impl,
    type_url,
    SerDeError,
};

//
// CreateEdge
//

#[derive(Debug, Clone, PartialEq)]
pub struct CreateEdge {
    pub source_uid: u64,
    pub dest_uid: u64,
    pub forward_edge_name: String,
    pub reverse_edge_name: String,
    pub source_node_type: String,
    pub dest_node_type: String,
}

impl TryFrom<CreateEdgeProto> for CreateEdge {
    type Error = SerDeError;

    fn try_from(response_proto: CreateEdgeProto) -> Result<Self, Self::Error> {
        Ok(CreateEdge {
            source_uid: response_proto.source_uid,
            dest_uid: response_proto.dest_uid,
            forward_edge_name: response_proto.forward_edge_name,
            reverse_edge_name: response_proto.reverse_edge_name,
            source_node_type: response_proto.source_node_type,
            dest_node_type: response_proto.dest_node_type,
        })
    }
}

impl From<CreateEdge> for CreateEdgeProto {
    fn from(response: CreateEdge) -> Self {
        CreateEdgeProto {
            source_uid: response.source_uid,
            dest_uid: response.dest_uid,
            forward_edge_name: response.forward_edge_name,
            reverse_edge_name: response.reverse_edge_name,
            source_node_type: response.source_node_type,
            dest_node_type: response.dest_node_type,
        }
    }
}

impl type_url::TypeUrl for CreateEdge {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.CloseLensResponse";
}

impl serde_impl::ProtobufSerializable for CreateEdge {
    type ProtobufMessage = CreateEdgeProto;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeleteEdge {
    pub source_uid: u64,
    pub dest_uid: u64,
    pub forward_edge_name: String,
    pub reverse_edge_name: String,
    pub source_node_type: String,
    pub dest_node_type: String,
}

impl TryFrom<DeleteEdgeProto> for DeleteEdge {
    type Error = SerDeError;

    fn try_from(response_proto: DeleteEdgeProto) -> Result<Self, Self::Error> {
        Ok(DeleteEdge {
            source_uid: response_proto.source_uid,
            dest_uid: response_proto.dest_uid,
            forward_edge_name: response_proto.forward_edge_name,
            reverse_edge_name: response_proto.reverse_edge_name,
            source_node_type: response_proto.source_node_type,
            dest_node_type: response_proto.dest_node_type,
        })
    }
}

impl From<DeleteEdge> for DeleteEdgeProto {
    fn from(response: DeleteEdge) -> Self {
        DeleteEdgeProto {
            source_uid: response.source_uid,
            dest_uid: response.dest_uid,
            forward_edge_name: response.forward_edge_name,
            reverse_edge_name: response.reverse_edge_name,
            source_node_type: response.source_node_type,
            dest_node_type: response.dest_node_type,
        }
    }
}

impl type_url::TypeUrl for DeleteEdge {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.DeleteEdge";
}

impl serde_impl::ProtobufSerializable for DeleteEdge {
    type ProtobufMessage = DeleteEdgeProto;
}

//
// CreateNode
//

#[derive(Debug, Clone, PartialEq)]
pub struct CreateNode {
    pub uid: u64,
    pub node_type: String,
}

impl TryFrom<CreateNodeProto> for CreateNode {
    type Error = SerDeError;

    fn try_from(response_proto: CreateNodeProto) -> Result<Self, Self::Error> {
        Ok(CreateNode {
            uid: response_proto.uid,
            node_type: response_proto.node_type,
        })
    }
}

impl From<CreateNode> for CreateNodeProto {
    fn from(response: CreateNode) -> Self {
        CreateNodeProto {
            uid: response.uid,
            node_type: response.node_type,
        }
    }
}

impl type_url::TypeUrl for CreateNode {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.CreateNode";
}

impl serde_impl::ProtobufSerializable for CreateNode {
    type ProtobufMessage = CreateNodeProto;
}

//
// DeleteNode
//

#[derive(Debug, Clone, PartialEq)]
pub struct DeleteNode {
    pub uid: u64,
    pub node_type: String,
}

impl TryFrom<DeleteNodeProto> for DeleteNode {
    type Error = SerDeError;

    fn try_from(response_proto: DeleteNodeProto) -> Result<Self, Self::Error> {
        Ok(DeleteNode {
            uid: response_proto.uid,
            node_type: response_proto.node_type,
        })
    }
}

impl From<DeleteNode> for DeleteNodeProto {
    fn from(response: DeleteNode) -> Self {
        DeleteNodeProto {
            uid: response.uid,
            node_type: response.node_type,
        }
    }
}

impl type_url::TypeUrl for DeleteNode {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.DeleteNode";
}

impl serde_impl::ProtobufSerializable for DeleteNode {
    type ProtobufMessage = DeleteNodeProto;
}

//
// Operation
//

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    CreateEdgeOperation(CreateEdge),
    DeleteEdgeOperation(DeleteEdge),
    CreateNodeOperation(CreateNode),
    DeleteNodeOperation(DeleteNode),
}

impl TryFrom<OperationProto> for Operation {
    type Error = SerDeError;

    fn try_from(response_proto: OperationProto) -> Result<Self, Self::Error> {
        match response_proto.graph_operations {
            Some(GraphOperationsProto::CreateEdgeOperation(e)) => {
                Ok(Operation::CreateEdgeOperation(e.try_into()?))
            }
            Some(GraphOperationsProto::DeleteEdgeOperation(e)) => {
                Ok(Operation::DeleteEdgeOperation(e.try_into()?))
            }
            Some(GraphOperationsProto::CreateNodeOperation(e)) => {
                Ok(Operation::CreateNodeOperation(e.try_into()?))
            }
            Some(GraphOperationsProto::DeleteNodeOperation(e)) => {
                Ok(Operation::DeleteNodeOperation(e.try_into()?))
            }
            _ => Err(SerDeError::UnknownVariant(
                "GraphOperationsProto.graphoperations",
            )),
        }
    }
}

impl From<Operation> for OperationProto {
    fn from(response_proto: Operation) -> Self {
        OperationProto {
            graph_operations: Some(match response_proto {
                Operation::CreateEdgeOperation(e) => {
                    GraphOperationsProto::CreateEdgeOperation(e.into())
                }
                Operation::DeleteEdgeOperation(e) => {
                    GraphOperationsProto::DeleteEdgeOperation(e.into())
                }
                Operation::CreateNodeOperation(e) => {
                    GraphOperationsProto::CreateNodeOperation(e.into())
                }
                Operation::DeleteNodeOperation(e) => {
                    GraphOperationsProto::DeleteNodeOperation(e.into())
                }
            }),
        }
    }
}

impl type_url::TypeUrl for Operation {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.Operation";
}

impl serde_impl::ProtobufSerializable for Operation {
    type ProtobufMessage = OperationProto;
}

//
// LensUpdate
//

#[derive(Debug, Clone, PartialEq)]
pub struct LensUpdate {
    pub operation: Operation,
    pub lens_type: String,
    pub lens_name: String,
    pub tenant_id: uuid::Uuid,
}

impl TryFrom<LensUpdateProto> for LensUpdate {
    type Error = SerDeError;

    fn try_from(response_proto: LensUpdateProto) -> Result<Self, Self::Error> {
        let tenant_id = response_proto
            .tenant_id
            .ok_or(Self::Error::MissingField("LensUpdateProto.tenant_id"))?
            .into();

        let operation = response_proto
            .operation
            .ok_or(SerDeError::MissingField("LensUpdateProto.operaton"))?
            .try_into()?;

        Ok(Self {
            operation: operation,
            lens_type: response_proto.lens_type,
            lens_name: response_proto.lens_name,
            tenant_id,
        })
    }
}

impl From<LensUpdate> for LensUpdateProto {
    fn from(response: LensUpdate) -> Self {
        LensUpdateProto {
            operation: Some(response.operation.into()),
            lens_type: response.lens_type,
            lens_name: response.lens_name,
            tenant_id: Some(response.tenant_id.into()),
        }
    }
}

impl type_url::TypeUrl for LensUpdate {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.LensUpdate";
}

impl serde_impl::ProtobufSerializable for LensUpdate {
    type ProtobufMessage = LensUpdateProto;
}
//
// LensSubscription
//

#[derive(Debug, Clone, PartialEq)]
pub struct LensSubscription {
    pub lens_type: String,
    pub lens_name: String,
    pub tenant_id: uuid::Uuid,
    pub update_id: u64,
}

impl TryFrom<LensSubscriptionProto> for LensSubscription {
    type Error = SerDeError;

    fn try_from(response_proto: LensSubscriptionProto) -> Result<Self, Self::Error> {
        let tenant_id = response_proto
            .tenant_id
            .ok_or(Self::Error::MissingField("LensSubscriptionProto.tenant_id"))?;

        Ok(LensSubscription {
            lens_type: response_proto.lens_type,
            lens_name: response_proto.lens_name,
            tenant_id: tenant_id.into(),
            update_id: response_proto.update_id,
        })
    }
}

impl From<LensSubscription> for LensSubscriptionProto {
    fn from(response: LensSubscription) -> Self {
        LensSubscriptionProto {
            lens_type: response.lens_type,
            lens_name: response.lens_name,
            tenant_id: Some(response.tenant_id.into()),
            update_id: response.update_id,
        }
    }
}

impl type_url::TypeUrl for LensSubscription {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.LensSubscription";
}

impl serde_impl::ProtobufSerializable for LensSubscription {
    type ProtobufMessage = LensSubscriptionProto;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubscribeToLensRequest {
    pub lens_subscription: LensSubscription,
}

impl TryFrom<SubscribeToLensRequestProto> for SubscribeToLensRequest {
    type Error = SerDeError;

    fn try_from(response_proto: SubscribeToLensRequestProto) -> Result<Self, Self::Error> {
        let lens_subscription = response_proto
            .lens_subscription
            .ok_or(Self::Error::MissingField("lens_subscription"))?
            .try_into()?;

        Ok(Self { lens_subscription })
    }
}

impl From<SubscribeToLensRequest> for SubscribeToLensRequestProto {
    fn from(response: SubscribeToLensRequest) -> Self {
        Self {
            lens_subscription: Some(response.lens_subscription.into()),
        }
    }
}

impl type_url::TypeUrl for SubscribeToLensRequest {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.LensSubscription";
}

impl serde_impl::ProtobufSerializable for SubscribeToLensRequest {
    type ProtobufMessage = SubscribeToLensRequestProto;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubscribeToLensResponse {
    pub operation: LensUpdate,
}

impl TryFrom<SubscribeToLensResponseProto> for SubscribeToLensResponse {
    type Error = SerDeError;

    fn try_from(response_proto: SubscribeToLensResponseProto) -> Result<Self, Self::Error> {
        let operation = response_proto
            .operation
            .ok_or(Self::Error::MissingField("operation"))?
            .try_into()?;

        Ok(Self { operation })
    }
}

impl From<SubscribeToLensResponse> for SubscribeToLensResponseProto {
    fn from(response: SubscribeToLensResponse) -> Self {
        Self {
            operation: Some(response.operation.into()),
        }
    }
}

impl type_url::TypeUrl for SubscribeToLensResponse {
    const TYPE_URL: &'static str =
        "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.SubscribeToLensResponse";
}

impl serde_impl::ProtobufSerializable for SubscribeToLensResponse {
    type ProtobufMessage = SubscribeToLensResponseProto;
}
