pub mod server;
pub mod client;

pub mod orgmanagementlib {
    tonic::include_proto!("orgmanagement");
}

pub mod create_db_conn;
pub mod organization_manager;

