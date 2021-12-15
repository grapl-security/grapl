pub mod client;
pub mod server;

pub mod plugin_registry {
    const PORT_ENV_VAR: &'static str = "GRAPL_PLUGIN_REGISTRY_PORT";

    pub fn get_socket_addr() -> Result<std::net::SocketAddr, std::net::AddrParseError> {
        let port = std::env::var(PORT_ENV_VAR).expect(PORT_ENV_VAR);
        return format!("0.0.0.0:{}", port).parse();
    }
}
