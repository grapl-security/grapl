use std::collections::HashMap;

use failure::Error;
use grapl_utils::rusoto_ext::dynamodb::GraplDynamoDbClientExt;
use rust_proto_new::graplinc::grapl::api::graph::v1beta1::{
    GraphDescription,
    IdentifiedGraph,
    IdentifiedNode,
    NodeDescription,
};
use tap::tap::TapOptional;

use crate::{
    dynamic_sessiondb::NodeDescriptionIdentifier,
    error::NodeIdentifierError,
};

/**
    The `NodeIdentifier` takes in graphs of previously unidentified nodes and
    performs identification based on the configured strategies for that node
    type.

    The strategies come in two variants:

    * [Session](`graplinc::grapl::api::graph::v1beta1::Session`) - strategy used
      for nodes with lifetimes (e.g. process, network connection)

    * [Static](`graplinc::grapl::api::graph::v1beta1::Static`) - strategy used
      for nodes with canonical and unique identifiers (e.g. aws events)
*/
#[derive(Clone)]
pub(crate) struct NodeIdentifier<D>
where
    D: GraplDynamoDbClientExt,
{
    dynamic_identifier: NodeDescriptionIdentifier<D>,
}

impl<D> NodeIdentifier<D>
where
    D: GraplDynamoDbClientExt,
{
    pub(crate) fn new(dynamic_identifier: NodeDescriptionIdentifier<D>) -> Self {
        Self { dynamic_identifier }
    }

    // todo: We should be yielding IdentifiedNode's here
    #[tracing::instrument(fields(node_key=?node.node_key), skip(self, node))]
    async fn attribute_node_key(&self, node: &NodeDescription) -> Result<IdentifiedNode, Error> {
        let new_node = self.dynamic_identifier.attribute_dynamic_node(node).await?;
        Ok(new_node.into())
    }

    /// Performs batch identification of unidentified nodes into identified
    /// nodes.
    ///
    /// A map of unidentified node keys to identified node keys will be returned
    /// in addition to the last error, if any, that occurred while identifying
    /// nodes.
    #[tracing::instrument(skip(self, unidentified_subgraph, identified_graph))]
    async fn identify_nodes(
        &self,
        unidentified_subgraph: &GraphDescription,
        identified_graph: &mut IdentifiedGraph,
    ) -> (HashMap<String, String>, Option<failure::Error>) {
        let mut identified_nodekey_map = HashMap::new();
        let mut attribution_failure = None;

        // new method
        for (unidentified_node_key, unidentified_node) in unidentified_subgraph.nodes.iter() {
            let identified_node = match self.attribute_node_key(unidentified_node).await {
                Ok(identified_node) => identified_node,
                Err(e) => {
                    tracing::warn!(
                        message="Failed to attribute node_key",
                        node_key=?unidentified_node_key,
                        error=?e
                    );
                    attribution_failure = Some(e);
                    continue;
                }
            };

            identified_nodekey_map.insert(
                unidentified_node_key.to_owned(),
                identified_node.clone_node_key(),
            );
            identified_graph.add_node(identified_node);
        }

        (identified_nodekey_map, attribution_failure)
    }

    /// Takes the edges in the `unidentified_graph` and inserts ones that can be
    /// properly identified into the `identified_graph` using the
    /// `identified_nodekey_map` returned from a previous node key
    /// identification process.
    #[tracing::instrument(skip(
        self,
        unidentified_subgraph,
        identified_graph,
        identified_nodekey_map
    ))]
    fn identify_edges(
        &self,
        unidentified_subgraph: &GraphDescription,
        identified_graph: &mut IdentifiedGraph,
        identified_nodekey_map: HashMap<String, String>,
    ) {
        let identified_node_edges = unidentified_subgraph.edges.iter()
            // filter out all edges for nodes that were not identified (also gets our from_key)
            .filter_map(|(from_key, edge_list)| {
                identified_nodekey_map.get(from_key)
                    .tap_none(|| tracing::warn!(
                        message = concat!(
                            "Could not get node_key mapping for from_key.",
                            "This means we can't identify the edge because we rely on the node to first be identified."
                        ),
                        from_key=?from_key,
                    ))
                    .map(|identified_from_key| (identified_from_key, edge_list))
            });

        // for each of the edges from identified nodes...
        for (identified_from_key, edge_list) in identified_node_edges {
            // map the to_key for each edge with the new, identified to_key
            let identified_edges = edge_list.edges.iter()
                .filter_map(|edge| {
                    // check if
                    identified_nodekey_map.get(&edge.to_node_key)
                        .tap_none(|| tracing::warn!(
                                message=concat!(
                                    "Could not get node_key mapping for to_key.",
                                    "This means we can't identify the edge because we rely on the node to first be identified."
                                ),
                                to_key=?edge.to_node_key
                            )
                        )
                        .map(|identified_to_edge| (identified_to_edge, &edge.edge_name))
                });

            // add all identified edges into the `identified_graph`
            for (identified_to_key, edge_name) in identified_edges {
                identified_graph.add_edge(
                    edge_name.to_owned(),
                    identified_from_key.to_owned(),
                    identified_to_key.to_owned(),
                );
            }
        }
    }

    #[tracing::instrument(skip(self, unidentified_subgraph))]
    pub(crate) async fn handle_event(
        &self,
        unidentified_subgraph: GraphDescription,
    ) -> Result<IdentifiedGraph, Result<(IdentifiedGraph, NodeIdentifierError), NodeIdentifierError>>
    {
        /*
           1. for the nodes in the unidentified graph, do the following
               1. identify each node
               2. record the old node key -> new node key mapping
               3. record which nodes failed to identify
               4. place the correctly identified nodes into a new, identified graph
           2. for the edges in the unidentified graph, do the following
               1. for both the 'from' and 'to' sections, look up the old-node-key -> new-node-key map to fetch updated values
                   1. if both are present, update edge and add it to the identified graph
                   2. if one or more are missing, warn with an appropriate error message and skip the edge
           3. if identified graph is empty, return as full error
           4. if any nodes failed to identify or any errors occurred, return as a partial error
           5. return as full graph
        */

        tracing::info!(
            message="Identifying a new graph",
            node_count=?unidentified_subgraph.nodes.len(),
            edge_count=?unidentified_subgraph.edges.len(),
        );

        if unidentified_subgraph.is_empty() {
            tracing::warn!("Received empty subgraph.");
            return Err(Err(NodeIdentifierError::EmptyGraph));
        }

        let mut identified_graph = IdentifiedGraph::new();

        let (identified_nodekey_map, attribution_failure) = self
            .identify_nodes(&unidentified_subgraph, &mut identified_graph)
            .await;

        tracing::info!(
            message="Performed node identification",
            total_edges=?unidentified_subgraph.nodes.len(),
            identified_edges=?identified_graph.nodes.len()
        );

        self.identify_edges(
            &unidentified_subgraph,
            &mut identified_graph,
            identified_nodekey_map,
        );

        tracing::info!(
            message = "Performed edge identification",
            total_edges = unidentified_subgraph.edges.len(),
            identified_edges = identified_graph.edges.len()
        );

        match attribution_failure {
            Some(_) if identified_graph.is_empty() => Err(Err(NodeIdentifierError::EmptyGraph)),
            Some(_) => {
                /* todo: error message is misleading. someone reading this would
                 * believe we identified a smaller number of nodes that actually
                 * identified (due to identities of nodes coalescing)
                 */
                tracing::warn!(
                    message = "Partial Success",
                    identified_nodes = identified_graph.nodes.len(),
                    identified_edges = identified_graph.edges.len(),
                );

                Err(Ok((
                    identified_graph,
                    NodeIdentifierError::AttributionFailure,
                )))
            }
            None => {
                tracing::info!("Identified all nodes");

                Ok(identified_graph)
            }
        }
    }
}
