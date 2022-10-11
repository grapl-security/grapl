use futures::future::join_all;
pub use rust_proto::graplinc::grapl::api::graph_query::v1beta1::comparators::StringCmp;
use rust_proto::graplinc::grapl::{
    api::graph_query::v1beta1::messages::{
        GraphQuery,
        GraphView,
    },
    common::v1beta1::types::Uid,
};

use crate::{
    node_query::{
        fetch_node_with_edges,
        NodeQueryError,
    },
    property_query::PropertyQueryExecutor,
    short_circuit::ShortCircuit,
    visited::Visited,
};

#[derive(thiserror::Error, Debug)]
pub enum GraphQueryError {
    #[error("Node query failed (uid: '{uid:?}'): {source}")]
    NodeQueryError { uid: Uid, source: NodeQueryError },
}

#[tracing::instrument(skip(graph_query, property_query_executor))]
pub async fn query_graph(
    graph_query: &GraphQuery,
    uid: Uid,
    tenant_id: uuid::Uuid,
    property_query_executor: PropertyQueryExecutor,
) -> Result<Option<(GraphView, Uid)>, GraphQueryError> {
    let mut query_handles = Vec::with_capacity(graph_query.node_property_queries.len());
    let x_query_short_circuiter = ShortCircuit::new();
    for node_query in graph_query.node_property_queries.values() {
        let property_query_executor = property_query_executor.clone();
        let node_query = node_query.clone();
        let x_query_short_circuiter = x_query_short_circuiter.clone();
        query_handles.push(async move {
            let visited = Visited::new();
            let mut root_query_uid = None;
            match fetch_node_with_edges(
                &node_query,
                graph_query,
                uid,
                tenant_id,
                property_query_executor,
                visited,
                x_query_short_circuiter.clone(),
                &mut root_query_uid,
            )
            .await
            {
                Ok(Some(g)) => {
                    x_query_short_circuiter.set_short_circuit();
                    Ok(Some((g, root_query_uid)))
                }
                Ok(None) => Ok(None),
                Err(e) => Err(GraphQueryError::NodeQueryError { uid, source: e }),
            }
        });
    }
    // todo: We don't need to join_all, we can stop polling the other futures
    //       once one of them matches
    //       try_select may work better
    for graph in join_all(query_handles).await {
        match graph {
            Ok(Some((graph, Some(root_uid)))) => return Ok(Some((graph, root_uid))),
            Ok(Some((_, None))) => {
                tracing::error!(
                    message = "Graph query matched without finding root_uid. This is a bug.",
                )
            }
            Ok(None) => continue,
            Err(e) => {
                tracing::error!(
                    message="Graph query failed",
                    error=?e,
                );
                return Err(e.into());
            }
        }
    }
    Ok(None)
}
