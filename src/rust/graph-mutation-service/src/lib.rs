#![allow(warnings)]

use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
use scylla::prepared_statement::PreparedStatement;
use scylla::Session;
use uuid::Uuid;
use rust_proto_new::graplinc::grapl::api::graph::v1beta1::Property;
use rust_proto_new::graplinc::grapl::api::graph_mutation::v1beta1::messages::{CreateEdgeRequest, CreateEdgeResponse, CreateNodeRequest, CreateNodeResponse, SetNodePropertyRequest, SetNodePropertyResponse, Uid};
use rust_proto_new::graplinc::grapl::api::graph_mutation::v1beta1::server::{GraphMutationApi, Status};

macro_rules! prepare_statement {
    ($tenant_statements:expr, $session:expr, $tenant_id:expr, $table_literal:ident) => {
        match $tenant_statements.entry($tenant_id) {
            Entry::Occupied(entry) => {
                Ok(entry.get().$table_literal.clone())
            }
            Entry::Vacant(entry) => {
                PreparedInsert::prepare($session, $tenant_id).await.map(|statement| {
                    entry.insert(statement).$table_literal.clone()
                })
            }
        }
    };
}

const MAX_I_64_TABLE_NAME: &str = "max_i64";
const MIN_I_64_TABLE_NAME: &str = "min_i64";
const IMM_I_64_TABLE_NAME: &str = "imm_i64";
const MAX_U_64_TABLE_NAME: &str = "max_u64";
const MIN_U_64_TABLE_NAME: &str = "min_u64";
const IMM_U_64_TABLE_NAME: &str = "imm_u64";
const IMM_STRING_TABLE_NAME: &str = "imm_string";

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

#[derive(Clone)]
struct PreparedInsert {
    pub max_i64: PreparedStatement,
    pub min_i64: PreparedStatement,
    pub imm_i64: PreparedStatement,
    pub max_u64: PreparedStatement,
    pub min_u64: PreparedStatement,
    pub imm_u64: PreparedStatement,
    pub imm_string: PreparedStatement,
}

// todo: Consider adding the node_type to the prepared statement
fn raw_property_prepare_statement(tenant_id: uuid::Uuid, table_name: &str) -> String {
    let tenant_urn = tenant_id.to_urn();
    format!(r"
            INSERT INTO tenant_keyspace_{tenant_urn}.{table_name} (uid, node_type, property_name, property_value)
            VALUES (?, ?, ?, ?)
            ")
}

impl PreparedInsert {

    /// Construct a PreparedInsert, with an internal PreparedStatement for each table for a tenant
    pub async fn prepare(scylla_client: &Session, tenant_id: uuid::Uuid) -> Result<Self, Box<dyn std::error::Error>> {
        let max_i64 = scylla_client.prepare(raw_property_prepare_statement(tenant_id, MAX_I_64_TABLE_NAME)).await?;
        let min_i64 = scylla_client.prepare(raw_property_prepare_statement(tenant_id, MIN_I_64_TABLE_NAME)).await?;
        let imm_i64 = scylla_client.prepare(raw_property_prepare_statement(tenant_id, IMM_I_64_TABLE_NAME)).await?;
        let max_u64 = scylla_client.prepare(raw_property_prepare_statement(tenant_id, MAX_U_64_TABLE_NAME)).await?;
        let min_u64 = scylla_client.prepare(raw_property_prepare_statement(tenant_id, MIN_U_64_TABLE_NAME)).await?;
        let imm_u64 = scylla_client.prepare(raw_property_prepare_statement(tenant_id, IMM_U_64_TABLE_NAME)).await?;
        let imm_string = scylla_client.prepare(raw_property_prepare_statement(tenant_id, IMM_STRING_TABLE_NAME)).await?;

        Ok(Self {
            max_i64,
            min_i64,
            imm_i64,
            max_u64,
            min_u64,
            imm_u64,
            imm_string,
        })
    }
}

#[derive(Clone)]
struct PreparedStatements {
    tenant_statements: DashMap<uuid::Uuid, PreparedInsert>,
}

impl PreparedStatements {
    pub async fn prepare_max_i64(&self, scylla_client: &Session, tenant_id: uuid::Uuid) -> Result<PreparedStatement, Box<dyn std::error::Error>> {
        prepare_statement!(self.tenant_statements, scylla_client,  tenant_id, max_i64)
    }
    pub async fn prepare_min_i64(&self, scylla_client: &Session, tenant_id: uuid::Uuid) -> Result<PreparedStatement, Box<dyn std::error::Error>> {
        prepare_statement!(self.tenant_statements, scylla_client,  tenant_id, min_i64)
    }

    pub async fn prepare_imm_i64(&self, scylla_client: &Session, tenant_id: uuid::Uuid) -> Result<PreparedStatement, Box<dyn std::error::Error>> {
        prepare_statement!(self.tenant_statements, scylla_client,  tenant_id, imm_i64)
    }
    pub async fn prepare_max_u64(&self, scylla_client: &Session, tenant_id: uuid::Uuid) -> Result<PreparedStatement, Box<dyn std::error::Error>> {
        prepare_statement!(self.tenant_statements, scylla_client,  tenant_id, max_u64)
    }
    pub async fn prepare_min_u64(&self, scylla_client: &Session, tenant_id: uuid::Uuid) -> Result<PreparedStatement, Box<dyn std::error::Error>> {
        prepare_statement!(self.tenant_statements, scylla_client,  tenant_id, min_u64)
    }
    pub async fn prepare_imm_u64(&self, scylla_client: &Session, tenant_id: uuid::Uuid) -> Result<PreparedStatement, Box<dyn std::error::Error>> {
        prepare_statement!(self.tenant_statements, scylla_client,  tenant_id, imm_u64)
    }
    pub async fn prepare_imm_string(&self, scylla_client: &Session, tenant_id: uuid::Uuid) -> Result<PreparedStatement, Box<dyn std::error::Error>> {
        prepare_statement!(self.tenant_statements, scylla_client,  tenant_id, imm_string)
    }
}

struct GraphMutationManager {
    scylla_client: Arc<Session>,
    prepared_statements: PreparedStatements,
}

impl GraphMutationManager {
    #[tracing::instrument(skip(self), err)]
    async fn upsert_max_integer(&self, tenant_keyspace: uuid::Uuid, uid: Uid, node_type: &str, property_name: &str, property_value: i64) -> Result<(), Box<dyn std::error::Error>> {
        // Create a prepared statement, and then execute it
        let mut statement = self.prepared_statements.prepare_max_i64(&self.scylla_client, tenant_keyspace).await?;

        statement.set_timestamp(Some(property_value));

        self.scylla_client.execute(
            &statement,
            (uid.as_i64(), node_type, property_name, property_value),
        ).await?;
        Ok(())
    }

    async fn upsert_min_integer(&self, tenant_keyspace: uuid::Uuid, uid: Uid, node_type: &str, property_name: &str, property_value: i64) -> Result<(), Box<dyn std::error::Error>> {
        // Create a prepared statement, and then execute it
        let mut statement = self.prepared_statements.prepare_min_i64(&self.scylla_client, tenant_keyspace).await?;

        statement.set_timestamp(Some(property_value * -1));

        self.scylla_client.execute(
            &statement,
            (uid.as_i64(), node_type, property_name, property_value),
        ).await?;
        Ok(())
    }

    async fn upsert_immutable_integer(&self, tenant_keyspace: uuid::Uuid, uid: Uid, node_type: &str, property_name: &str, property_value: i64) -> Result<(), Box<dyn std::error::Error>> {
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

    async fn create_node(&self, request: CreateNodeRequest) -> Result<CreateNodeResponse, Self::Error> {
        // todo: Blocked on uid-allocator
        todo!()
    }

    async fn set_node_property(&self, request: SetNodePropertyRequest) -> Result<SetNodePropertyResponse, Self::Error> {
        let SetNodePropertyRequest { tenant_id, uid, node_type, property_name, property } = request;
        match property.property {
            Property::IncrementOnlyUint(property) => {
                todo!()
                // Ok(self.upsert_max_integer(property.prop).await?)
            }
            Property::DecrementOnlyUint(property) => {
                todo!()
                // Ok(self.upsert_decrement_only_uint(property.prop).await?)
            }
            Property::ImmutableUint(property) => {
                todo!()
                // Ok(self.upsert_immutable_uint(property.prop).await?)
            }
            Property::IncrementOnlyInt(property) => {
                self.upsert_max_integer(
                    tenant_id.into(),
                    uid,
                    &node_type.value,
                    &property_name.value,
                    property.prop,
                ).await?;
            }
            Property::DecrementOnlyInt(property) => {
                self.upsert_min_integer(
                    tenant_id.into(),
                    uid,
                    &node_type.value,
                    &property_name.value,
                    property.prop,
                ).await?;
            }
            Property::ImmutableInt(property) => {
                self.upsert_immutable_integer(
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

    async fn create_edge(&self, request: CreateEdgeRequest) -> Result<CreateEdgeResponse, Self::Error> {
        todo!()
    }
}