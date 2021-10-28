pub mod orgmanagementlib {
    tonic::include_proto!("orgmanagementproto");
}

pub mod server;
pub mod create_db_conn;
pub mod organization_manager;

// pub mod client;