use std::sync::Arc;

use async_trait::async_trait;
use rust_proto::{
    graplinc::grapl::api::db_schema_manager::v1beta1::{
        messages::{
            DeployGraphSchemasRequest,
            DeployGraphSchemasResponse,
        },
        server::DbSchemaManagerApi,
    },
    protocol::status::Status,
};
use scylla::{
    transport::errors::QueryError,
    Session,
};

#[derive(thiserror::Error, Debug)]
pub enum DbSchemaManagerError {
    #[error("Scylla Error {0}")]
    ScyllaError(#[from] QueryError),
}

impl From<DbSchemaManagerError> for Status {
    fn from(error: DbSchemaManagerError) -> Self {
        match error {
            DbSchemaManagerError::ScyllaError(error) => {
                Status::unknown(format!("Scylla Error: {}", error).as_str())
            }
        }
    }
}

#[derive(Clone)]
pub struct DbSchemaManager {
    scylla_client: Arc<Session>,
}

#[async_trait]
impl DbSchemaManagerApi for DbSchemaManager {
    type Error = DbSchemaManagerError;

    async fn deploy_graph_schemas(
        &self,
        request: DeployGraphSchemasRequest,
    ) -> Result<DeployGraphSchemasResponse, Self::Error> {
        let DeployGraphSchemasRequest { tenant_id } = request;
        let session = self.scylla_client.as_ref();

        let tenant_ks = format!("tenant_keyspace_{}", tenant_id.urn());

        session.query(
            format!(
                r"CREATE KEYSPACE IF NOT EXISTS {tenant_ks} WITH REPLICATION = {{'class' : 'SimpleStrategy', 'replication_factor' : 1}};"
            ),
            &[]
        ).await?;

        let property_table_names = [
            ("immutable_strings", "text"),
            ("immutable_i64", "bigint"),
            ("max_i64", "bigint"),
            ("min_i64", "bigint"),
            ("immutable_u64", "bigint"),
            ("max_u64", "bigint"),
            ("min_u64", "bigint"),
        ];

        for (table_name, value_type) in property_table_names.into_iter() {
            session
                .query(
                    format!(
                        r"CREATE TABLE IF NOT EXISTS {tenant_ks}.{table_name} (
                    uid bigint,
                    populated_field text,
                    value {value_type},
                    PRIMARY KEY (uid, populated_field)
                )"
                    ),
                    &(),
                )
                .await?;
        }

        session
            .query(
                format!(
                    r"CREATE TABLE IF NOT EXISTS {tenant_ks}.edges (
                    src_uid bigint,
                    dst_uid bigint,
                    edge_name text,
                    PRIMARY KEY (src_uid, edge_name, dst_uid)
                )"
                ),
                &(),
            )
            .await?;

        session.await_schema_agreement().await?;

        Ok(DeployGraphSchemasResponse {})
    }
}
