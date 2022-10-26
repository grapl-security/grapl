use std::{
    collections::HashMap,
    fmt::Debug,
};

use grapl_tracing::SetupTracingError;
use kafka::StreamProcessorError;
use rust_proto::graplinc::grapl::{
    api::{
        client::ClientError,
        graph::v1beta1::{
            IdentifiedEdge,
            IdentifiedGraph,
            Property,
        },
        graph_mutation::v1beta1::{
            client::GraphMutationClient,
            messages::{
                CreateEdgeRequest,
                MutationRedundancy,
                SetNodePropertyRequest,
            },
        },
        plugin_sdk::analyzers::v1beta1::messages::{
            EdgeUpdate,
            Int64PropertyUpdate,
            StringPropertyUpdate,
            UInt64PropertyUpdate,
            Update,
        },
    },
    common::v1beta1::types::{
        EdgeName,
        NodeType,
        PropertyName,
        Uid,
    },
    pipeline::v1beta1::Envelope,
};

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum GraphMergerError {
    #[error("unexpected error")]
    Unexpected(String),

    #[error("error processing event {0}")]
    StreamProcessorError(#[from] kafka::StreamProcessorError),

    #[error("missing environment variable {0}")]
    MissingEnvironmentVariable(#[from] std::env::VarError),

    #[error("kafka configuration error {0}")]
    KafkaConfigurationError(#[from] kafka::ConfigurationError),

    #[error("failed to configure tracing {0}")]
    SetupTracingError(#[from] SetupTracingError),

    #[error("gRPC client error {0}")]
    GrpcClientError(#[from] ClientError),
}

impl From<GraphMergerError> for kafka::StreamProcessorError {
    fn from(graph_merger_error: GraphMergerError) -> Self {
        kafka::StreamProcessorError::EventHandlerError(graph_merger_error.to_string())
    }
}

impl From<&GraphMergerError> for kafka::StreamProcessorError {
    fn from(graph_merger_error: &GraphMergerError) -> Self {
        kafka::StreamProcessorError::EventHandlerError(graph_merger_error.to_string())
    }
}

#[derive(Clone)]
pub struct GraphMerger {
    graph_mutation_client: GraphMutationClient,
}

impl GraphMerger {
    pub fn new(graph_mutation_client: GraphMutationClient) -> Self {
        Self {
            graph_mutation_client,
        }
    }

    #[tracing::instrument(skip(self, event))]
    pub async fn handle_event(
        mut self,
        event: Result<(tracing::Span, Envelope<IdentifiedGraph>), StreamProcessorError>,
    ) -> Vec<Result<Envelope<Update>, GraphMergerError>> {
        match event {
            Ok((span, envelope)) => {
                let _guard = span.enter();
                let tenant_id = envelope.tenant_id();
                let trace_id = envelope.trace_id();
                let event_source_id = envelope.event_source_id();
                let subgraph = envelope.inner_message();

                tracing::debug!("received kafka message");

                if subgraph.is_empty() {
                    tracing::warn!("Attempted to merge empty subgraph. Short circuiting.");
                    return vec![];
                }

                tracing::info!(
                    message = "handling new subgraph",
                    nodes =? subgraph.nodes.len(),
                    edges =? subgraph.edges.len(),
                );

                let mut updates = Vec::with_capacity(subgraph.nodes.len() + subgraph.edges.len());

                let node_types: HashMap<Uid, String> = subgraph
                    .nodes
                    .iter()
                    .map(|(uid, n)| (*uid, n.node_type.clone()))
                    .collect();
                let nodes = subgraph.nodes;
                let edges = subgraph.edges;

                for node in nodes.into_values() {
                    for (prop_name, prop_value) in node.properties {
                        let update =
                            property_to_update(node.uid, prop_name.clone(), &prop_value.property);

                        let response = self
                            .graph_mutation_client
                            .set_node_property(SetNodePropertyRequest {
                                tenant_id,
                                node_type: NodeType {
                                    value: node.node_type.clone(),
                                },
                                uid: node.uid,
                                property_name: PropertyName { value: prop_name },
                                property: prop_value,
                            })
                            .await;

                        match response {
                            Ok(set_node_property_response) => {
                                if let MutationRedundancy::True =
                                    set_node_property_response.mutation_redundancy
                                {
                                    continue;
                                } else {
                                    updates.push(Ok(Envelope::new(
                                        tenant_id,
                                        trace_id,
                                        event_source_id,
                                        update,
                                    )));
                                }
                            }
                            Err(e) => updates.push(Err(e.into())),
                        }
                    }
                }

                for edge_list in edges.into_values() {
                    for edge in edge_list.edges {
                        let IdentifiedEdge {
                            to_uid,
                            from_uid,
                            edge_name,
                        } = edge;
                        let response = self
                            .graph_mutation_client
                            .create_edge(CreateEdgeRequest {
                                tenant_id,
                                edge_name: EdgeName {
                                    value: edge_name.clone(),
                                },
                                from_uid,
                                to_uid,
                                source_node_type: NodeType {
                                    value: node_types[&from_uid].clone(),
                                },
                            })
                            .await;

                        match response {
                            Ok(create_edge_response) => {
                                if let MutationRedundancy::True =
                                    create_edge_response.mutation_redundancy
                                {
                                    continue;
                                } else {
                                    updates.push(Ok(Envelope::new(
                                        tenant_id,
                                        trace_id,
                                        event_source_id,
                                        Update::Edge(EdgeUpdate {
                                            src_uid: from_uid,
                                            dst_uid: to_uid,
                                            forward_edge_name: EdgeName {
                                                value: edge_name.clone(),
                                            },
                                            reverse_edge_name: EdgeName {
                                                value: edge_name.clone(),
                                            },
                                        }),
                                    )));
                                }
                            }
                            Err(e) => updates.push(Err(e.into())),
                        }
                    }
                }

                updates
            }
            Err(e) => vec![Err(e.into())],
        }
    }
}

fn property_to_update(uid: Uid, property_name: String, property: &Property) -> Update {
    match property {
        Property::IncrementOnlyUintProp(_)
        | Property::DecrementOnlyUintProp(_)
        | Property::ImmutableUintProp(_) => Update::Uint64Property(UInt64PropertyUpdate {
            uid,
            property_name: PropertyName {
                value: property_name,
            },
        }),
        Property::IncrementOnlyIntProp(_)
        | Property::DecrementOnlyIntProp(_)
        | Property::ImmutableIntProp(_) => Update::Int64Property(Int64PropertyUpdate {
            uid,
            property_name: PropertyName {
                value: property_name,
            },
        }),
        Property::ImmutableStrProp(_) => Update::StringProperty(StringPropertyUpdate {
            uid,
            property_name: PropertyName {
                value: property_name,
            },
        }),
    }
}
