use dashmap::DashMap;
use scylla::{
    batch::Consistency,
    prepared_statement::PreparedStatement,
    query::Query,
    transport::errors::QueryError,
    Session,
};


#[derive(thiserror::Error, Debug)]
pub enum PreparedStatementsError {
    #[error("QueryError: {0}")]
    QueryError(#[from] QueryError),
}

#[derive(Clone)]
pub(crate) struct PreparedInsert {
    pub edges: PreparedStatement,
}


fn raw_edge_prepare_statement(tenant_id: uuid::Uuid) -> String {
    let tenant_urn = tenant_id.urn();
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

impl PreparedInsert {
    /// Construct a PreparedInsert, with an internal PreparedStatement for each table for a tenant
    pub(crate) async fn prepare(
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<Self, PreparedStatementsError> {
        let edges = prepare_upsert(scylla_client, raw_edge_prepare_statement(tenant_id)).await?;

        Ok(Self {
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

    pub(crate) async fn prepare_edge(
        &self,
        scylla_client: &Session,
        tenant_id: uuid::Uuid,
    ) -> Result<PreparedStatement, PreparedStatementsError> {
        match self.tenant_statements.entry(tenant_id) {
            dashmap::mapref::entry::Entry::Occupied(entry) => {
                Ok(entry.get().edges.clone())
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                crate::prepared_statements::PreparedInsert::prepare(scylla_client, tenant_id)
                    .await
                    .map(|statement| entry.insert(statement).edges.clone())
            }
        }
        // prepare_statement!(self.tenant_statements, scylla_client, tenant_id, edges)
    }
}
