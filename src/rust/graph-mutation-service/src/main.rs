use dgraph_tonic::{
    Client as DgraphClient,
    Status,
};
use graph_mutation_service_lib::reverse_resolver::{
    ReverseEdgeResolver,
    ReverseEdgeResolverError,
};
use grapl_config::env_helpers::FromEnv;
use grapl_graph_descriptions::graph_mutation_service::graph_mutation_rpc_server::GraphMutationRpcServer;
pub use grapl_graph_descriptions::{
    graph_mutation_service::{
        graph_mutation_rpc_server::GraphMutationRpc,
        *,
    },
    *,
};
use grapl_observe::metric_reporter::MetricReporter;
use rusoto_dynamodb::DynamoDbClient;
use tonic::{
    transport::Server,
    Request,
    Response,
};

use crate::{
    mutations::{
        edge_mutation::EdgeUpsertGenerator,
        node_mutation::NodeUpsertGenerator,
    },
    upsert_manager::{
        UpsertManager,
        UpsertManagerError,
    },
};

pub mod mutations;
pub mod upsert_manager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (env, _guard) = grapl_config::init_grapl_env!();

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
    .set_serving::<GraphMutationRpcServer<GraphMutationService>>()
    .await;



    let mg_alphas = grapl_config::mg_alphas();
    let dgraph_client =
        std::sync::Arc::new(DgraphClient::new(mg_alphas).expect("Failed to create dgraph client."));

    let addr = "0.0.0.0:5500".parse().unwrap();
    let service = GraphMutationService {
        reverse_edge_resolver: ReverseEdgeResolver::new(
            DynamoDbClient::from_env(),
            MetricReporter::new(&env.service_name),
            256,
        ),
        dgraph_client,
    };

    println!("GraphMutationRpcServer listening on {}", addr);

    Server::builder()
        .add_service(health_service)
        .add_service(GraphMutationRpcServer::new(service))
        .serve(addr)
        .await?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum GraphMutationError {
    #[error("SetNode missing field")]
    SetNodeMissingField { missing_field: &'static str },
    #[error("SetEdge missing field")]
    SetEdgeMissingField { missing_field: &'static str },
    #[error("Unexpected number of reverse edges")]
    UnexpectedNumberOfReverseEdges {
        forward_edge: String,
        reverse_edge_count: usize,
    },
    #[error("UpsertManagerError")]
    UpsertManagerError(#[from] UpsertManagerError),
    #[error("ReverseEdgeResolverError")]
    ReverseEdgeResolverError(#[from] ReverseEdgeResolverError),
}

impl From<GraphMutationError> for Status {
    fn from(_err: GraphMutationError) -> Status {
        dbg!(&_err);
        unimplemented!()
    }
}

pub struct GraphMutationService {
    reverse_edge_resolver: ReverseEdgeResolver,
    dgraph_client: std::sync::Arc<DgraphClient>,
}

impl GraphMutationService {
    async fn _set_node(&self, node: IdentifiedNode) -> Result<u64, GraphMutationError> {
        let mut upsert_manager = UpsertManager {
            dgraph_client: self.dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
            edge_upsert_generator: EdgeUpsertGenerator::default(),
        };
        let uid = upsert_manager.upsert_node(&node).await?;
        Ok(uid)
    }

    async fn _set_edge(&self, edge: Edge) -> Result<(u64, u64), GraphMutationError> {
        let mut reversed = self
            .reverse_edge_resolver
            .resolve_reverse_edges(vec![edge.clone()])
            .await?;
        if reversed.len() > 1 {
            return Err(GraphMutationError::UnexpectedNumberOfReverseEdges {
                forward_edge: edge.edge_name.to_owned(),
                reverse_edge_count: reversed.len(),
            });
        }
        let reversed = reversed.remove(0);

        let mut upsert_manager = UpsertManager {
            dgraph_client: self.dgraph_client.clone(),
            node_upsert_generator: NodeUpsertGenerator::default(),
            edge_upsert_generator: EdgeUpsertGenerator::default(),
        };
        let (src_uid, dst_uid) = upsert_manager.upsert_edge(edge, reversed).await?;
        Ok((src_uid, dst_uid))
    }
}

#[tonic::async_trait]
impl GraphMutationRpc for GraphMutationService {
    #[tracing::instrument(
        fields(
            remote_address = ? request.remote_addr(),
            trace_id =? uuid::Uuid::new_v4(),
        ),
        skip(self, request),
        err,
    )]
    async fn set_node(
        &self,
        request: Request<SetNodeRequest>,
    ) -> Result<Response<SetNodeResult>, Status> {
        let request = request.into_inner();

        let node = match request.node {
            Some(node) => node,
            None => {
                return Err(GraphMutationError::SetNodeMissingField {
                    missing_field: "node",
                }
                .into())
            }
        };
        let node_uid = self._set_node(node).await?;
        Ok(tonic::Response::new(SetNodeResult {
            rpc_result: Some(set_node_result::RpcResult::Set(SetNodeSuccess { node_uid })),
        }))
    }

    #[tracing::instrument(
        fields(
            remote_address = ? request.remote_addr(),
            trace_id =? uuid::Uuid::new_v4(),
        ),
        skip(self, request),
        err,
    )]
    async fn set_edge(
        &self,
        request: Request<SetEdgeRequest>,
    ) -> Result<Response<SetEdgeResult>, Status> {
        let request = request.into_inner();

        let edge = match request.edge {
            Some(edge) => edge,
            None => {
                return Err(GraphMutationError::SetEdgeMissingField {
                    missing_field: "edge",
                }
                .into())
            }
        };

        let (src_uid, dst_uid) = self._set_edge(edge).await?;

        Ok(tonic::Response::new(SetEdgeResult {
            rpc_result: Some(set_edge_result::RpcResult::Set(SetEdgeSuccess {
                src_uid,
                dst_uid,
            })),
        }))
    }
}
