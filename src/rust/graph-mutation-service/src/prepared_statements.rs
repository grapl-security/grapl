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

#[derive(thiserror::Error, Debug)]
pub enum PreparedStatementsError {
    #[error("QueryError: {0}")]
    QueryError(#[from] QueryError),
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
    pub node_type: PreparedStatement,
    pub edges: PreparedStatement,
}

fn raw_node_typeprepare_statement(tenant_id: uuid::Uuid) -> String {
    let tenant_urn = tenant_id.to_urn();
    format!(
        r"
            INSERT INTO tenant_keyspace_{tenant_urn}.node_type (uid, node_type)
            VALUES (?, ?)
            "
    )
}

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
                source_node_type,
                destination_node_type,
            )
            VALUES (?, ?, ?, ?, ?, ?)
            ",
    )
}

// TODO: This is all useless, the CachingSession does this all for us -_-
// todo we should take the `uid` and `property_name` for the statements
// and cache the statements so that we can use token based load balancing
impl PreparedInsert {
    /// Construct a PreparedInsert, with an internal PreparedStatement for each table for a tenant
    pub(crate) async fn prepare(
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<Self, PreparedStatementsError> {
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

        let node_type =
            prepare_upsert(scylla_client, raw_node_typeprepare_statement(tenant_id)).await?;

        let edges = prepare_upsert(scylla_client, raw_edge_prepare_statement(tenant_id)).await?;

        Ok(Self {
            max_i64,
            min_i64,
            imm_i64,
            max_u64,
            min_u64,
            imm_u64,
            imm_string,
            node_type,
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
    pub(crate) fn new() -> Self {
        Self {
            tenant_statements: DashMap::new(),
        }
    }

    pub(crate) async fn prepare_max_i64(
        &self,
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<PreparedStatement, PreparedStatementsError> {
        prepare_statement!(self.tenant_statements, scylla_client, tenant_id, max_i64)
    }

    pub(crate) async fn prepare_min_i64(
        &self,
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<PreparedStatement, PreparedStatementsError> {
        prepare_statement!(self.tenant_statements, scylla_client, tenant_id, min_i64)
    }

    pub(crate) async fn prepare_imm_i64(
        &self,
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<PreparedStatement, PreparedStatementsError> {
        prepare_statement!(self.tenant_statements, scylla_client, tenant_id, imm_i64)
    }

    pub(crate) async fn prepare_max_u64(
        &self,
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<PreparedStatement, PreparedStatementsError> {
        prepare_statement!(self.tenant_statements, scylla_client, tenant_id, max_u64)
    }

    pub(crate) async fn prepare_min_u64(
        &self,
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<PreparedStatement, PreparedStatementsError> {
        prepare_statement!(self.tenant_statements, scylla_client, tenant_id, min_u64)
    }

    pub(crate) async fn prepare_imm_u64(
        &self,
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<PreparedStatement, PreparedStatementsError> {
        prepare_statement!(self.tenant_statements, scylla_client, tenant_id, imm_u64)
    }

    pub(crate) async fn prepare_imm_string(
        &self,
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<PreparedStatement, PreparedStatementsError> {
        prepare_statement!(self.tenant_statements, scylla_client, tenant_id, imm_string)
    }

    pub(crate) async fn prepare_node_type(
        &self,
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<PreparedStatement, PreparedStatementsError> {
        prepare_statement!(self.tenant_statements, scylla_client, tenant_id, node_type)
    }

    pub(crate) async fn prepare_edge(
        &self,
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<PreparedStatement, PreparedStatementsError> {
        prepare_statement!(self.tenant_statements, scylla_client, tenant_id, edges)
    }
}
