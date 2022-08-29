use std::collections::{
    HashMap,
    HashSet,
};

use failure::{
    bail,
    Error,
};
use rusoto_dynamodb::DynamoDb;
use rust_proto::graplinc::grapl::{
    api::graph::v1beta1::{
        GraphDescription,
        IdentifiedGraph,
        IdentifiedNode,
        NodeDescription,
        Session,
        Static,
        Strategy,
    },
    common::v1beta1::types::Uid,
};
use serde::{
    Deserialize,
    Serialize,
};
use sha2::{
    Digest,
    Sha256,
};
use rust_proto::graplinc::grapl::api::graph_mutation::v1beta1::client::GraphMutationClient;

use crate::{
    sessiondb::SessionDb,
    sessions::UnidSession,
    StaticMappingDb,
};

#[derive(Debug, Clone)]
pub(crate) struct NodeDescriptionIdentifier<D>
where
    D: DynamoDb,
{
    dyn_session_db: SessionDb<D>,
    graph_mutation_client: GraphMutationClient,
    static_mapping_db: StaticMappingDb<D>,
    should_guess: bool,
}

impl<D> NodeDescriptionIdentifier<D>
where
    D: DynamoDb,
{
    pub fn new(
        dyn_session_db: SessionDb<D>,
        graph_mutation_client: GraphMutationClient,
        static_mapping_db: StaticMappingDb<D>,
        should_guess: bool,
    ) -> Self {
        Self {
            dyn_session_db,
            graph_mutation_client,
            static_mapping_db,
            should_guess,
        }
    }

    #[tracing::instrument(skip(self, node, strategy), err)]
    async fn primary_session_key(
        &self,
        node: &mut NodeDescription,
        strategy: &Session,
    ) -> Result<String, Error> {
        let mut primary_key = String::with_capacity(32);

        if strategy.primary_key_requires_asset_id {
            panic!("asset_id resolution is currently not supported")
        }

        for prop_name in &strategy.primary_key_properties {
            let prop_val = node.properties.get(prop_name);

            match prop_val {
                Some(val) => primary_key.push_str(&val.to_string()),
                None => bail!(format!(
                    "Node is missing required property {} for identity",
                    prop_name
                )),
            }
        }

        // Push node type, as a natural partition
        primary_key.push_str(&node.node_type);

        Ok(primary_key)
    }

    #[tracing::instrument(skip(self, strategy), err)]
    pub(crate) async fn attribute_dynamic_session(
        &self,
        tenant_id: uuid::Uuid,
        node: &NodeDescription,
        strategy: &Session,
    ) -> Result<IdentifiedNode, Error> {
        let mut attributed_node = node.clone();

        let primary_key = self
            .primary_session_key(&mut attributed_node, strategy)
            .await?;

        let created_time = strategy.create_time;
        let last_seen_time = strategy.last_seen_time;

        let unid = match (created_time != 0, last_seen_time != 0) {
            (true, _) => UnidSession {
                pseudo_key: primary_key,
                node_type: attributed_node.node_type,
                timestamp: created_time,
                is_creation: true,
            },
            (_, true) => UnidSession {
                pseudo_key: primary_key,
                node_type: attributed_node.node_type,
                timestamp: last_seen_time,
                is_creation: false,
            },
            _ => bail!(
                "Terminating sessions not yet supported: {:?} {:?}",
                node.properties,
                &strategy,
            ),
        };

        let session_id = self
            .dyn_session_db
            .handle_unid_session(tenant_id, unid, &self.graph_mutation_client, self.should_guess)
            .await?;

        Ok(IdentifiedNode {
            properties: node.properties.clone(),
            uid: session_id,
            node_type: node.node_type.to_string(),
        })
    }

    #[tracing::instrument(skip(self, node, strategy), err)]
    pub(crate) async fn attribute_static_mapping(
        &self,
        tenant_id: uuid::Uuid,
        node: &NodeDescription,
        strategy: &Static,
    ) -> Result<IdentifiedNode, Error> {
        let uid = self
            .static_mapping_db
            .map_unid(tenant_id, node, strategy)
            .await?;

        Ok(IdentifiedNode {
            properties: node.properties.clone(),
            uid,
            node_type: node.node_type.to_string(),
        })
    }

    #[tracing::instrument(skip(self, node), err)]
    pub(crate) async fn attribute_dynamic_node(
        &self,
        tenant_id: uuid::Uuid,
        node: &NodeDescription,
    ) -> Result<IdentifiedNode, Error> {
        let strategy = &node.id_strategy[0];

        match strategy.strategy {
            Strategy::Session(ref strategy) => {
                tracing::info!("Attributing dynamic node via session");
                self.attribute_dynamic_session(tenant_id, node, strategy)
                    .await
            }
            Strategy::Static(ref strategy) => {
                tracing::info!("Attributing dynamic node via static mapping");
                self.attribute_static_mapping(tenant_id, node, strategy)
                    .await
            }
        }
    }
}
