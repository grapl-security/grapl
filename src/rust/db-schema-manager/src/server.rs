use std::sync::Arc;
use std::time::Duration;

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
use tokio::net::TcpListener;
use rust_proto::graplinc::grapl::api::db_schema_manager::v1beta1::server::DbSchemaManagerServer;
use rust_proto::protocol::healthcheck::HealthcheckStatus;
use crate::config::DbSchemaManagerServiceConfig;

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

        let tenant_ks = format!("tenant_keyspace_{}", tenant_id.simple());

        session.query(
            format!(
                r"CREATE KEYSPACE IF NOT EXISTS {tenant_ks} WITH REPLICATION = {{'class' : 'SimpleStrategy', 'replication_factor' : 1}};"
            ),
            &[]
        ).await?;

        let property_table_names = [
            ("immutable_strings", "text"),
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
                    r"CREATE TABLE IF NOT EXISTS {tenant_ks}.node_type (
                        uid bigint,
                        node_type text,
                        PRIMARY KEY (uid, node_type)
                    )"
                ),
                &(),
            )
            .await?;
        session
            .query(
                format!(
                    r"CREATE TABLE IF NOT EXISTS {tenant_ks}.edges (
                        source_uid bigint,
                        destination_uid bigint,
                        f_edge_name text,
                        r_edge_name text,
                        PRIMARY KEY (source_uid, f_edge_name, destination_uid)
                    )"
                ),
                &(),
            )
            .await?;

        session.await_schema_agreement().await?;

        Ok(DeployGraphSchemasResponse {})
    }
}

#[tracing::instrument(skip(config), err)]
pub async fn exec_service(config: DbSchemaManagerServiceConfig) -> Result<(), Box<dyn std::error::Error>> {
    let graph_db_config = config.graph_db_config;

    let addr = config.db_schema_manager_bind_address;
    tracing::info!(
        message="Starting Db Schema Manager Service",
        addr=?addr,
        graph_db_addresses=?graph_db_config.graph_db_addresses,
    );

    let plugin_registry = DbSchemaManager {
        scylla_client: Arc::new(graph_db_config.connect().await?),
    };

    let healthcheck_polling_interval_ms = 5000; // TODO: un-hardcode
    let (server, _shutdown_tx) = DbSchemaManagerServer::new(
        plugin_registry,
        TcpListener::bind(addr).await?,
        || async { Ok(HealthcheckStatus::Serving) }, // FIXME: this is garbage
        Duration::from_millis(healthcheck_polling_interval_ms),
    );
    tracing::info!(
        message = "starting gRPC server",
        socket_address = %addr,
    );

    Ok(server.serve().await?)
}
