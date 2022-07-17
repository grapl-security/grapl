use std::net::SocketAddr;

#[derive(clap::Parser, Clone, Debug)]
pub struct LensSubscriptionServiceConfig {
    #[clap(long, env)]
    pub lens_subscription_service_bind_address: SocketAddr,
    #[clap(long, env)]
    pub lens_subscription_service_healthcheck_polling_interval_ms: u64,
    #[clap(flatten)]
    pub db_config: LensSubscriptionDbConfig,
}

#[derive(clap::Parser, Clone, Debug)]
pub struct LensSubscriptionDbConfig {
    #[clap(long, env)]
    pub lens_subscription_service_db_hostname: String,
    #[clap(long, env)]
    pub lens_subscription_service_db_port: u16,
    #[clap(long, env)]
    pub lens_subscription_service_db_username: String,
    #[clap(long, env)]
    pub lens_subscription_service_db_password: String,
}


