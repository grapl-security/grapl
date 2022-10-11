use crate::{
    graplinc::grapl::common::v1beta1::types::{
        Uid,
    },
    protobufs::graplinc::grapl::api::graph_query_proxy::v1beta1 as proto,
    SerDeError,
};

// Re-export the Response types
pub use crate::graplinc::grapl::api::graph_query::{
    QueryGraphFromUidResponse,
    QueryGraphWithUidResponse;
}

#[derive(Debug, Clone)]
pub struct QueryGraphWithUidRequest {
    pub node_uid: Uid,
    pub graph_query: GraphQuery,
}

impl TryFrom<proto::QueryGraphWithUidRequest> for QueryGraphWithUidRequest {
    type Error = SerDeError;

    fn try_from(value: proto::QueryGraphWithUidRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            node_uid: value
                .node_uid
                .ok_or(SerDeError::MissingField("node_uid"))?
                .try_into()?,
            graph_query: value
                .graph_query
                .ok_or(SerDeError::MissingField("node_query"))?
                .try_into()?,
        })
    }
}

impl From<QueryGraphWithUidRequest> for proto::QueryGraphWithUidRequest {
    fn from(value: QueryGraphWithUidRequest) -> Self {
        Self {
            node_uid: Some(value.node_uid.into()),
            graph_query: Some(value.graph_query.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryGraphFromUidRequest {
    pub node_uid: Uid,
    pub graph_query: GraphQuery,
}

impl TryFrom<proto::QueryGraphFromUidRequest> for QueryGraphFromUidRequest {
    type Error = SerDeError;

    fn try_from(value: proto::QueryGraphFromUidRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            node_uid: value
                .node_uid
                .ok_or(SerDeError::MissingField("node_uid"))?
                .try_into()?,
            graph_query: value
                .graph_query
                .ok_or(SerDeError::MissingField("graph_query"))?
                .try_into()?,
        })
    }
}

impl From<QueryGraphFromUidRequest> for proto::QueryGraphFromUidRequest {
    fn from(value: QueryGraphFromUidRequest) -> Self {
        Self {
            node_uid: Some(value.node_uid.into()),
            graph_query: Some(value.graph_query.into()),
        }
    }
}