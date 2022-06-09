use crate::{
    protobufs::graplinc::grapl::api::lens_subscription::v1beta1::{
        CreateEdge as CreateEdgeProto,
    },
    type_url,
    serde_impl,
    SerDeError,
};

// message CreateEdge {
// uint64 source_uid = 1;
// uint64 dest_uid = 2;
// string forward_edge_name = 3;
// string reverse_edge_name = 4;
// string source_node_type = 5;
// string dest_node_type = 6;
// }

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
        "graplsecurity.com/TODO";
}

impl serde_impl::ProtobufSerializable for CreateEdge {
    type ProtobufMessage = CreateEdgeProto;
}

// impl type_url::TypeUrl for CloseLensResponse {
//     const TYPE_URL: &'static str =
//         "graplsecurity.com/graplinc.grapl.api.lens_manager.v1beta1.CloseLensResponse";
// }
//
// impl serde_impl::ProtobufSerializable for CloseLensResponse {
//     type ProtobufMessage = CloseLensResponseProto;
// }
