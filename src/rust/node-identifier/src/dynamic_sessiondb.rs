use failure::{
    bail,
    Error,
};
use grapl_graph_descriptions::graph_description::{
    id_strategy::Strategy,
    *,
};
use sha2::{
    Digest,
    Sha256,
};

use crate::{
    sessiondb::SessionDb,
    sessions::UnidSession,
};
use tap::Tap;
use itertools::{
    Either,
    Itertools
};
use crate::sessiondb::UnidSessionNode;
use grapl_utils::rusoto_ext::dynamodb::GraplDynamoDbClientExt;

#[derive(Debug, Clone)]
pub struct NodeDescriptionIdentifier<D>
where
    D: GraplDynamoDbClientExt,
{
    dyn_session_db: SessionDb<D>,
    should_guess: bool,
}

impl<D> NodeDescriptionIdentifier<D>
where
    D: GraplDynamoDbClientExt,
{
    pub fn new(
        dyn_session_db: SessionDb<D>,
        should_guess: bool,
    ) -> Self {
        Self {
            dyn_session_db,
            should_guess,
        }
    }

    /// Creates an identifier that can be used to track sessions for dynamic nodes or used directly
    /// as the node_key for a statically identifiable node.
    fn get_strategy_key(
        &self,
        node: &NodeDescription,
        strategy_properties: &Vec<String>,
        requires_asset_id: bool
    ) -> Result<String, Error> {
        if requires_asset_id {
            panic!("asset_id resolution is currently not supported")
        }

        let mut hasher = Sha256::new();

        // first, let's sort the properties, so we get a consistent ordering for hashing
        let sorted_key_properties = strategy_properties.clone()
            .tap_mut(|v| v.sort());

        for prop_name in sorted_key_properties {
            match node.properties.get(&prop_name) {
                Some(prop_val) => hasher.update(prop_val.to_string().as_bytes()),
                None => bail!(
                    "Node is missing required property {} for identity",
                    prop_name
                ),
            }
        }

        hasher.update(node.node_type.as_bytes());

        Ok(hex::encode(hasher.finalize()))
    }

    /// Splits nodes out that are able to get strategy keys with nodes that fail to do so
    /// This can eventually go away, one day, when getting strategy keys can no longer fail
    fn get_strategy_keys_for_session_nodes<'a>(
        &self,
        session_nodes: Vec<SessionNodeDescription<'a>>
    ) -> (
        Vec<(SessionNodeDescription<'a>, String)>,
        Vec<Result<AttributedNode, Error>>
    ) {
        session_nodes.into_iter()
            .map(|node_desc| {
                let SessionNodeDescription(node, strategy) = &node_desc;

                let strategy_key = self.get_strategy_key(
                    node,
                    &strategy.primary_key_properties,
                    strategy.primary_key_requires_asset_id
                );

                (node_desc, strategy_key)
            })
            .partition_map(|(node_desc, strategy_key)| {
                match strategy_key {
                    Ok(strategy_key) => Either::Left((node_desc, strategy_key)),
                    Err(e) => Either::Right(Result::<AttributedNode, _>::Err(e))
                }
            })
    }

    /// Takes a collection of [`SessionNodeDescription`]s and converts them into unid session nodes
    ///
    /// This first gets the strategy key for each node
    fn get_unid_nodes_from_session_nodes(
        &self,
        session_nodes: Vec<SessionNodeDescription<'_>>
    ) -> (
        Vec<UnidSessionNode>,
        Vec<Result<AttributedNode, Error>>
    ) {
        let (
            nodes_with_strategy_keys,
            failed_to_fetch_keys
        ): (Vec<_>, Vec<_>) = self.get_strategy_keys_for_session_nodes(session_nodes);

        let (unid_sessions, unsupported_session_type): (Vec<_>, Vec<_>) = nodes_with_strategy_keys.into_iter()
            .partition_map(|(SessionNodeDescription(node_desc, strategy), strategy_key)| {
                let created_time = strategy.create_time;
                let last_seen_time = strategy.last_seen_time;

                let unid_session = match (created_time != 0, last_seen_time != 0) {
                    (true, _) => UnidSession {
                        pseudo_key: strategy_key,
                        timestamp: created_time,
                        is_creation: true,
                    },
                    (_, true) => UnidSession {
                        pseudo_key: strategy_key,
                        timestamp: last_seen_time,
                        is_creation: false,
                    },
                    _ => {
                        let error = failure::err_msg(format!(
                            "Terminating sessions not yet supported: {:?} {:?}",
                            node_desc.properties,
                            strategy,
                        ));

                        return Either::Right(Result::<AttributedNode, _>::Err(error));
                    },
                };

                Either::Left(UnidSessionNode::new(node_desc.clone(), unid_session))
            });

        let error_cases = unsupported_session_type.into_iter()
            .chain(failed_to_fetch_keys)
            .collect();

        (unid_sessions, error_cases)
    }

    async fn identify_session_nodes(
        &self,
        session_nodes: Vec<SessionNodeDescription<'_>>
    ) -> Vec<Result<AttributedNode, Error>> {
        let (
            unid_sessions,
            unid_errors
        ): (Vec<_>, Vec<_>) = self.get_unid_nodes_from_session_nodes(session_nodes);

        let identified_node_results = self.dyn_session_db
            .identify_unid_session_nodes(unid_sessions, self.should_guess)
            .await;

        let identified_nodes: Vec<_> = match identified_node_results {
            Ok(result) => {
                result.into_iter()
                    .map(|node| Ok(node))
                    .collect()
            },
            Err(error) => vec![Err(error)]
        };

        identified_nodes.into_iter()
            .chain(unid_errors)
            .collect()
    }

    /// Takes a collection of [`StaticNodeDescription`]s (which pair a [`NodeDescription`] and their [`Static`] strategy)
    /// and attempts to perform static identification on them, returning a vec of the results.
    fn identify_static_nodes(
        &self,
        static_nodes: Vec<StaticNodeDescription>
    ) -> Vec<Result<AttributedNode, Error>> {
        //  for each node
        //      1. attempt to get the strategy key
        //      2. if successful, assign it as the node_key for the node
        static_nodes.into_iter()
            .map(|StaticNodeDescription(static_node, static_strategy)| {
                // try to get the node key for the static strategy
                self.get_strategy_key(
                    &static_node,
                    &static_strategy.primary_key_properties,
                    static_strategy.primary_key_requires_asset_id
                ).and_then(|static_node_key| {
                    // set the node_key onto our node
                    let statically_identified_node = static_node.clone().tap_mut(|node| node.set_key(static_node_key));

                    Ok(AttributedNode::new(statically_identified_node, static_node.clone_node_key()))
                })
            }).collect()
    }

    /// Takes a collection of [`NodeDescription`]s and attempts to identify them. If a node is identified,
    /// it's old node_key will be returned alongside it.
    pub async fn attribute_nodes(
        &self,
        nodes: Vec<&NodeDescription>
    ) -> Vec<Result<AttributedNode, Error>> {
        let (
            statically_identifiable_nodes,
            dynamically_identifiable_nodes
        ): (Vec<_>, Vec<_>) = nodes.iter()
            .partition_map(|node| {
                let strategy = &node.id_strategy[0];

                match strategy.strategy.as_ref().unwrap() {
                    Strategy::Static(static_strategy) => Either::Left(StaticNodeDescription(node, static_strategy)),
                    Strategy::Session(session_strategy) => Either::Right(SessionNodeDescription(node, session_strategy))
                }
            });

        let statically_identified_node_results = self.identify_static_nodes(statically_identifiable_nodes);
        let dynamically_identified_node_results = self.identify_session_nodes(dynamically_identifiable_nodes).await;

        statically_identified_node_results.into_iter()
            .chain(dynamically_identified_node_results)
            .collect()
    }
}

pub struct AttributedNode {
    pub attributed_node_description: IdentifiedNode,
    pub previous_node_key: String
}

impl AttributedNode {
    pub(crate) fn new(attributed_node_description: NodeDescription, previous_node_key: String) -> Self {
        Self {
            attributed_node_description: attributed_node_description.into(),
            previous_node_key
        }
    }
}

struct StaticNodeDescription<'a>(&'a NodeDescription, &'a Static);
struct SessionNodeDescription<'a>(&'a NodeDescription, &'a Session);