use rust_proto_new::graplinc::grapl::api::graph::v1beta1::Property;
use rust_proto_new::graplinc::grapl::api::graph_mutation::v1beta1::messages::{CreateEdgeRequest, CreateEdgeResponse, CreateNodeRequest, CreateNodeResponse, SetNodePropertyRequest, SetNodePropertyResponse, Uid};
use rust_proto_new::graplinc::grapl::api::graph_mutation::v1beta1::server::GraphMutationApi;
use rust_proto_new::protocol::status::Status;
use std::sync::Arc;
use scylla::Session;
use crate::prepared_statements::PreparedStatements;

#[derive(thiserror::Error, Debug)]
enum GraphMutationManagerError {
    #[error("Unknown")]
    Error(#[from] Box<dyn std::error::Error>)
}

impl Into<Status> for GraphMutationManagerError {
    fn into(self) -> Status {
        Status::internal(
            self.to_string(),
        )
    }
}

struct GraphMutationManager {
    scylla_client: Arc<Session>,
    prepared_statements: PreparedStatements,
}

impl GraphMutationManager {
    #[tracing::instrument(skip(self), err)]
    async fn upsert_max_u64(&self, tenant_keyspace: uuid::Uuid, uid: Uid, node_type: &str, property_name: &str, property_value: u64) -> Result<(), Box<dyn std::error::Error>> {
        let property_value = property_value as i64;
        // Create a prepared statement, and then execute it
        let mut statement = self.prepared_statements.prepare_max_u64(&self.scylla_client, tenant_keyspace).await?;

        statement.set_timestamp(Some(property_value));

        self.scylla_client.execute(
            &statement,
            (uid.as_i64(), node_type, property_name, property_value),
        ).await?;
        Ok(())
    }

    async fn upsert_min_u64(&self, tenant_keyspace: uuid::Uuid, uid: Uid, node_type: &str, property_name: &str, property_value: u64) -> Result<(), Box<dyn std::error::Error>> {
        let property_value = property_value as i64;
        // Create a prepared statement, and then execute it
        let mut statement = self.prepared_statements.prepare_min_u64(&self.scylla_client, tenant_keyspace).await?;

        statement.set_timestamp(Some(property_value * -1));

        self.scylla_client.execute(
            &statement,
            (uid.as_i64(), node_type, property_name, property_value),
        ).await?;
        Ok(())
    }

    async fn upsert_immutable_u64(&self, tenant_keyspace: uuid::Uuid, uid: Uid, node_type: &str, property_name: &str, property_value: u64) -> Result<(), Box<dyn std::error::Error>> {
        let property_value = property_value as i64;
        // todo: We should only prepare statements once
        let mut statement = self.prepared_statements.prepare_imm_u64(&self.scylla_client, tenant_keyspace).await?;

        // Immutable values can never be overwritten
        statement.set_timestamp(Some(1i64));

        self.scylla_client.execute(
            &statement,
            (uid.as_i64(), node_type, property_name, property_value),
        ).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self), err)]
    async fn upsert_max_i64(&self, tenant_keyspace: uuid::Uuid, uid: Uid, node_type: &str, property_name: &str, property_value: i64) -> Result<(), Box<dyn std::error::Error>> {
        // Create a prepared statement, and then execute it
        let mut statement = self.prepared_statements.prepare_max_i64(&self.scylla_client, tenant_keyspace).await?;

        statement.set_timestamp(Some(property_value));

        self.scylla_client.execute(
            &statement,
            (uid.as_i64(), node_type, property_name, property_value),
        ).await?;
        Ok(())
    }

    async fn upsert_min_i64(&self, tenant_keyspace: uuid::Uuid, uid: Uid, node_type: &str, property_name: &str, property_value: i64) -> Result<(), Box<dyn std::error::Error>> {
        // Create a prepared statement, and then execute it
        let mut statement = self.prepared_statements.prepare_min_i64(&self.scylla_client, tenant_keyspace).await?;

        statement.set_timestamp(Some(property_value * -1));

        self.scylla_client.execute(
            &statement,
            (uid.as_i64(), node_type, property_name, property_value),
        ).await?;
        Ok(())
    }

    async fn upsert_immutable_i64(&self, tenant_keyspace: uuid::Uuid, uid: Uid, node_type: &str, property_name: &str, property_value: i64) -> Result<(), Box<dyn std::error::Error>> {
        // todo: We should only prepare statements once
        let mut statement = self.prepared_statements.prepare_imm_i64(&self.scylla_client, tenant_keyspace).await?;

        // Immutable values can never be overwritten
        statement.set_timestamp(Some(1i64));

        self.scylla_client.execute(
            &statement,
            (uid.as_i64(), node_type, property_name, property_value),
        ).await?;
        Ok(())
    }

    async fn upsert_immutable_string(&self, tenant_keyspace: uuid::Uuid, uid: Uid, node_type: &str, property_name: &str, property_value: String) -> Result<(), Box<dyn std::error::Error>> {
        // todo: Should we only prepare statements once?
        let mut statement = self.prepared_statements.prepare_imm_string(&self.scylla_client, tenant_keyspace).await?;

        // Immutable values can never be overwritten
        statement.set_timestamp(Some(1i64));

        self.scylla_client.execute(
            &statement,
            (uid.as_i64(), node_type, property_name, property_value),
        ).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl GraphMutationApi for GraphMutationManager {
    type Error = GraphMutationManagerError;

    /// Create Node allocates a new node in the graph, returning the uid of the new node.
    async fn create_node(&self, _request: CreateNodeRequest) -> Result<CreateNodeResponse, Self::Error> {
        // todo: Blocked on uid-allocator
        todo!()
    }

    /// SetNodeProperty will update the property of the node with the given uid.
    /// If the node does not exist it will be created.
    async fn set_node_property(&self, request: SetNodePropertyRequest) -> Result<SetNodePropertyResponse, Self::Error> {
        let SetNodePropertyRequest { tenant_id, uid, node_type, property_name, property } = request;
        match property.property {
            Property::IncrementOnlyUint(property) => {
                self.upsert_max_u64(
                    tenant_id.into(),
                    uid,
                    &node_type.value,
                    &property_name.value,
                    property.prop,
                ).await?;
            }
            Property::DecrementOnlyUint(property) => {
                self.upsert_min_u64(
                    tenant_id.into(),
                    uid,
                    &node_type.value,
                    &property_name.value,
                    property.prop,
                ).await?;
            }
            Property::ImmutableUint(property) => {
                self.upsert_immutable_u64(
                    tenant_id.into(),
                    uid,
                    &node_type.value,
                    &property_name.value,
                    property.prop,
                ).await?;
            }
            Property::IncrementOnlyInt(property) => {
                self.upsert_max_i64(
                    tenant_id.into(),
                    uid,
                    &node_type.value,
                    &property_name.value,
                    property.prop,
                ).await?;
            }
            Property::DecrementOnlyInt(property) => {
                self.upsert_min_i64(
                    tenant_id.into(),
                    uid,
                    &node_type.value,
                    &property_name.value,
                    property.prop,
                ).await?;
            }
            Property::ImmutableInt(property) => {
                self.upsert_immutable_i64(
                    tenant_id.into(),
                    uid,
                    &node_type.value,
                    &property_name.value,
                    property.prop,
                ).await?;
            }
            Property::ImmutableStr(property) => {
                self.upsert_immutable_string(
                    tenant_id.into(),
                    uid,
                    &node_type.value,
                    &property_name.value,
                    property.prop,
                ).await?;
            }
        };

        Ok(SetNodePropertyResponse {
            was_redundant: false
        })
    }

    /// CreateEdge will create an edge with the name edge_name between the nodes
    /// that have the given uids. It will also create the reverse edge.
    async fn create_edge(&self, _request: CreateEdgeRequest) -> Result<CreateEdgeResponse, Self::Error> {
        todo!()
    }
}
