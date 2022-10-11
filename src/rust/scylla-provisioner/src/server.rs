use std::{
    sync::{
        atomic::AtomicBool,
        Arc,
    },
    time::Duration,
};

use async_trait::async_trait;
use rust_proto::{
    graplinc::grapl::api::scylla_provisioner::v1beta1::{
        messages as native,
        server::{
            ScyllaProvisionerApi,
            ScyllaProvisionerServer,
        },
    },
    protocol::{
        healthcheck::HealthcheckStatus,
        status::Status,
    },
};
use scylla::{
    transport::errors::QueryError,
    Session,
};
use tokio::net::TcpListener;

use crate::{
    config::ScyllaProvisionerServiceConfig,
    table_names::{
        IMM_I_64_TABLE_NAME,
        IMM_STRING_TABLE_NAME,
        IMM_U_64_TABLE_NAME,
        MAX_I_64_TABLE_NAME,
        MAX_U_64_TABLE_NAME,
        MIN_I_64_TABLE_NAME,
        MIN_U_64_TABLE_NAME,
    },
};

#[derive(thiserror::Error, Debug)]
pub enum ScyllaProvisionerError {
    #[error("Scylla Error {0}")]
    ScyllaError(#[from] QueryError),
}

impl From<ScyllaProvisionerError> for Status {
    fn from(error: ScyllaProvisionerError) -> Self {
        match error {
            ScyllaProvisionerError::ScyllaError(error) => Status::unknown(error.to_string()),
        }
    }
}

#[derive(Clone)]
pub struct ScyllaProvisioner {
    scylla_client: Arc<Session>,
    already_provisioned: Arc<AtomicBool>,
}

#[async_trait]
impl ScyllaProvisionerApi for ScyllaProvisioner {
    type Error = ScyllaProvisionerError;

    async fn provision_graph_for_tenant(
        &self,
        request: native::ProvisionGraphForTenantRequest,
    ) -> Result<native::ProvisionGraphForTenantResponse, Self::Error> {
        if self
            .already_provisioned
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            return Ok(native::ProvisionGraphForTenantResponse {});
        }
        let native::ProvisionGraphForTenantRequest { tenant_id: _ } = request;
        let session = self.scylla_client.as_ref();

        session.query(
            r"CREATE KEYSPACE IF NOT EXISTS tenant_graph_ks WITH REPLICATION = {{'class' : 'SimpleStrategy', 'replication_factor' : 1}};",
            &[],
        ).await?;

        let property_table_names = [
            (IMM_STRING_TABLE_NAME, "text"),
            (MAX_I_64_TABLE_NAME, "bigint"),
            (MIN_I_64_TABLE_NAME, "bigint"),
            (IMM_I_64_TABLE_NAME, "bigint"),
            (MAX_U_64_TABLE_NAME, "bigint"),
            (MIN_U_64_TABLE_NAME, "bigint"),
            (IMM_U_64_TABLE_NAME, "bigint"),
        ];

        for (table_name, value_type) in property_table_names.into_iter() {
            session
                .query(
                    format!(
                        r"CREATE TABLE IF NOT EXISTS tenant_graph_ks.{table_name} (
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
                "CREATE TABLE IF NOT EXISTS tenant_graph_ks.node_type (
                        uid bigint,
                        node_type text,
                        PRIMARY KEY (uid, node_type)
                    )",
                &(),
            )
            .await?;
        session
            .query(
                r"CREATE TABLE IF NOT EXISTS tenant_graph_ks.edges (
                        source_uid bigint,
                        destination_uid bigint,
                        f_edge_name text,
                        r_edge_name text,
                        PRIMARY KEY (source_uid, f_edge_name, destination_uid)
                    )",
                &(),
            )
            .await?;

        session.await_schema_agreement().await?;
        self.already_provisioned
            .store(true, std::sync::atomic::Ordering::SeqCst);

        Ok(native::ProvisionGraphForTenantResponse {})
    }
}

#[tracing::instrument(skip(config), err)]
pub async fn exec_service(
    config: ScyllaProvisionerServiceConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let graph_db_config = config.graph_db_config;

    let addr = config.scylla_provisioner_bind_address;
    tracing::info!(
        message="Starting Db Schema Manager Service",
        addr=?addr,
        graph_db_addresses=?graph_db_config.graph_db_addresses,
    );

    let plugin_registry = ScyllaProvisioner {
        scylla_client: Arc::new(graph_db_config.connect().await?),
        already_provisioned: Arc::new(AtomicBool::new(false)),
    };

    let healthcheck_polling_interval_ms = 5000; // TODO: un-hardcode
    let (server, _shutdown_tx) = ScyllaProvisionerServer::new(
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
