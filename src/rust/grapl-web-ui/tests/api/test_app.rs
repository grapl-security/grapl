#![cfg(feature = "integration_tests")]

/// Represents an instance of our web server intended for use with
/// API integration tests.
///
/// Only API routes are supported. Paths, such as "/index.html" are not
/// supported because the frontend assets are not expected to be present
/// where ever these tests are being ran.
pub struct TestApp {
    pub bind_address: String,
    client: awc::Client,
}

async fn run_until_stopped(server: actix_web::dev::Server) -> Result<(), std::io::Error> {
    server.await
}

impl TestApp {
    pub fn spawn() -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = grapl_web_ui::Config::from_env()?;

        // Overwrite default bind address
        // let OS choose available port
        let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
        let bind_address = listener.local_addr()?.to_string();

        config.listener = listener;

        let server = grapl_web_ui::run(config)?;
        // move to backround
        let _ = tokio::spawn(run_until_stopped(server));

        Ok(Self {
            bind_address,
            client: awc::Client::default(),
        })
    }

    pub fn post(&self, path: &str) -> awc::ClientRequest {
        let bind_address = self.bind_address.as_str();
        self.client
            .post(format!("http://{bind_address}/{path}"))
            .insert_header(("content-type", "application/json"))
    }
}
