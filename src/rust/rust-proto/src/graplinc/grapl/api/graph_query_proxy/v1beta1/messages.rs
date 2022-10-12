// Re-export the Response types
use crate::{
    graplinc::grapl::{
        api::graph_query::v1beta1::messages as graph_query_messages,
        common::v1beta1::types::Uid,
    },
    protobufs::graplinc::grapl::api::graph_query_proxy::v1beta1 as proto,
    SerDeError,
};

#[derive(Debug, Clone)]
pub struct QueryGraphWithUidRequest {
    pub node_uid: Uid,
    pub graph_query: graph_query_messages::GraphQuery,
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
pub struct QueryGraphWithUidResponse {
    pub maybe_match: graph_query_messages::MaybeMatchWithUid,
}

impl TryFrom<proto::QueryGraphWithUidResponse> for QueryGraphWithUidResponse {
    type Error = SerDeError;
    fn try_from(value: proto::QueryGraphWithUidResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            maybe_match: value
                .maybe_match
                .ok_or(SerDeError::MissingField("maybe_match"))?
                .try_into()?,
        })
    }
}

impl From<QueryGraphWithUidResponse> for proto::QueryGraphWithUidResponse {
    fn from(value: QueryGraphWithUidResponse) -> Self {
        Self {
            maybe_match: Some(value.maybe_match.into()),
        }
    }
}

// Convert from a Graph Query response to a Graph Query Proxy response.
impl From<graph_query_messages::QueryGraphWithUidResponse> for QueryGraphWithUidResponse {
    fn from(other: graph_query_messages::QueryGraphWithUidResponse) -> Self {
        Self {
            maybe_match: other.maybe_match,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryGraphFromUidRequest {
    pub node_uid: Uid,
    pub graph_query: graph_query_messages::GraphQuery,
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

#[derive(Debug, Clone)]
pub struct QueryGraphFromUidResponse {
    pub matched_graph: Option<graph_query_messages::GraphView>,
}

impl TryFrom<proto::QueryGraphFromUidResponse> for QueryGraphFromUidResponse {
    type Error = SerDeError;
    fn try_from(value: proto::QueryGraphFromUidResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            matched_graph: value.matched_graph.map(|g| g.try_into()).transpose()?,
        })
    }
}

impl From<QueryGraphFromUidResponse> for proto::QueryGraphFromUidResponse {
    fn from(value: QueryGraphFromUidResponse) -> Self {
        Self {
            matched_graph: value.matched_graph.map(Into::into),
        }
    }
}

// Convert from a Graph Query response to a Graph Query Proxy response.
impl From<graph_query_messages::QueryGraphFromUidResponse> for QueryGraphFromUidResponse {
    fn from(other: graph_query_messages::QueryGraphFromUidResponse) -> Self {
        Self {
            matched_graph: other.matched_graph,
        }
    }
}
