use dashmap::DashMap;
use scylla::{
    batch::Consistency,
    prepared_statement::PreparedStatement,
    query::Query,
    transport::errors::QueryError,
    Session,
};

use crate::table_names::{
    IMM_I_64_TABLE_NAME,
    IMM_STRING_TABLE_NAME,
    IMM_U_64_TABLE_NAME,
    MAX_I_64_TABLE_NAME,
    MAX_U_64_TABLE_NAME,
    MIN_I_64_TABLE_NAME,
    MIN_U_64_TABLE_NAME,
};

macro_rules! prepare_statement {
    ($tenant_statements:expr, $session:expr, $tenant_id:expr, $table_literal:ident) => {
        match $tenant_statements.entry($tenant_id) {
            dashmap::mapref::entry::Entry::Occupied(entry) => {
                Ok(entry.get().$table_literal.clone())
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                crate::prepared_statements::PreparedInsert::prepare($session, $tenant_id)
                    .await
                    .map(|statement| entry.insert(statement).$table_literal.clone())
            }
        }
    };
}

#[derive(Clone)]
pub(crate) struct PreparedInsert {
    pub max_i64: PreparedStatement,
    pub min_i64: PreparedStatement,
    pub imm_i64: PreparedStatement,
    pub max_u64: PreparedStatement,
    pub min_u64: PreparedStatement,
    pub imm_u64: PreparedStatement,
    pub imm_string: PreparedStatement,
    pub edges: PreparedStatement,
}

// todo: Consider adding the node_type to the prepared statement
fn raw_property_prepare_statement(tenant_id: uuid::Uuid, table_name: &str) -> String {
    let tenant_urn = tenant_id.to_urn();
    format!(
        r"
            INSERT INTO tenant_keyspace_{tenant_urn}.{table_name} (uid, node_type, property_name, property_value)
            VALUES (?, ?, ?, ?)
            "
    )
}

fn raw_edge_prepare_statement(tenant_id: uuid::Uuid) -> String {
    let tenant_urn = tenant_id.to_urn();
    format!(
        r"
            INSERT INTO tenant_keyspace_{tenant_urn}.edges (
                source_uid,
                f_edge_name,
                r_edge_name,
                destination_uid,
            )
            VALUES (?, ?, ?, ?)
            ",
    )
}

impl PreparedInsert {
    /// Construct a PreparedInsert, with an internal PreparedStatement for each table for a tenant
    pub(crate) async fn prepare(
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let max_i64 = prepare_upsert(
            scylla_client,
            raw_property_prepare_statement(tenant_id, MAX_I_64_TABLE_NAME),
        )
        .await?;

        let min_i64 = prepare_upsert(
            scylla_client,
            raw_property_prepare_statement(tenant_id, MIN_I_64_TABLE_NAME),
        )
        .await?;

        let imm_i64 = prepare_upsert(
            scylla_client,
            raw_property_prepare_statement(tenant_id, IMM_I_64_TABLE_NAME),
        )
        .await?;

        let max_u64 = prepare_upsert(
            scylla_client,
            raw_property_prepare_statement(tenant_id, MAX_U_64_TABLE_NAME),
        )
        .await?;

        let min_u64 = prepare_upsert(
            scylla_client,
            raw_property_prepare_statement(tenant_id, MIN_U_64_TABLE_NAME),
        )
        .await?;

        let imm_u64 = prepare_upsert(
            scylla_client,
            raw_property_prepare_statement(tenant_id, IMM_U_64_TABLE_NAME),
        )
        .await?;

        let imm_string = prepare_upsert(
            scylla_client,
            raw_property_prepare_statement(tenant_id, IMM_STRING_TABLE_NAME),
        )
        .await?;

        let edges = prepare_upsert(scylla_client, raw_edge_prepare_statement(tenant_id)).await?;

        Ok(Self {
            max_i64,
            min_i64,
            imm_i64,
            max_u64,
            min_u64,
            imm_u64,
            imm_string,
            edges,
        })
    }
}

async fn prepare_upsert(
    scylla_client: &Session,
    query: impl Into<Query>,
) -> Result<PreparedStatement, QueryError> {
    let mut prepared = scylla_client.prepare(query).await?;

    prepared.set_is_idempotent(true);
    prepared.set_consistency(Consistency::Quorum);

    Ok(prepared)
}

#[derive(Clone)]
pub(crate) struct PreparedStatements {
    tenant_statements: DashMap<uuid::Uuid, PreparedInsert>,
}

impl PreparedStatements {
    pub(crate) async fn prepare_max_i64(
        &self,
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<PreparedStatement, Box<dyn std::error::Error>> {
        prepare_statement!(self.tenant_statements, scylla_client, tenant_id, max_i64)
    }

    pub(crate) async fn prepare_min_i64(
        &self,
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<PreparedStatement, Box<dyn std::error::Error>> {
        prepare_statement!(self.tenant_statements, scylla_client, tenant_id, min_i64)
    }

    pub(crate) async fn prepare_imm_i64(
        &self,
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<PreparedStatement, Box<dyn std::error::Error>> {
        prepare_statement!(self.tenant_statements, scylla_client, tenant_id, imm_i64)
    }

    pub(crate) async fn prepare_max_u64(
        &self,
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<PreparedStatement, Box<dyn std::error::Error>> {
        prepare_statement!(self.tenant_statements, scylla_client, tenant_id, max_u64)
    }

    pub(crate) async fn prepare_min_u64(
        &self,
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<PreparedStatement, Box<dyn std::error::Error>> {
        prepare_statement!(self.tenant_statements, scylla_client, tenant_id, min_u64)
    }

    pub(crate) async fn prepare_imm_u64(
        &self,
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<PreparedStatement, Box<dyn std::error::Error>> {
        prepare_statement!(self.tenant_statements, scylla_client, tenant_id, imm_u64)
    }

    pub(crate) async fn prepare_imm_string(
        &self,
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<PreparedStatement, Box<dyn std::error::Error>> {
        prepare_statement!(self.tenant_statements, scylla_client, tenant_id, imm_string)
    }

    pub(crate) async fn prepare_edge(
        &self,
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<PreparedStatement, Box<dyn std::error::Error>> {
        match self.tenant_statements.entry(tenant_id) {
            dashmap::mapref::entry::Entry::Occupied(entry) => Ok(entry.get().edges.clone()),
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                PreparedInsert::prepare(scylla_client, tenant_id)
                    .await
                    .map(|statement| entry.insert(statement).edges.clone())
            }
        }
    }
}
