mod test_user;

use test_user::TestUser;

use crate::config::TestConfig;

/// Represents an instance of our web server intended for use with
/// API integration tests.
///
/// Only API routes are supported. Paths, such as "/index.html" are not
/// supported because the frontend assets are not expected to be present
/// where ever these tests are being ran.
pub struct TestApp {
    pub endpoint_url: url::Url,
    pub client: reqwest::Client,
    pub test_user: TestUser,
}

impl TestApp {
    /// Initialize a new instance of TestApp.
    ///
    /// This provisions a new test user to use for authentication.
    pub async fn init() -> eyre::Result<Self> {
        let config = TestConfig::from_env()?;

        let endpoint_url = config.endpoint_address;
        println!("Initializing test for endpoint URL: {}", endpoint_url);

        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .cookie_store(true)
            .build()?;

        // create a test user entry in the database
        let test_user = TestUser::new();
        test_user
            .store(&config.user_auth_table_name, &config.dynamodb_client)
            .await?;

        // Wait for Nomad/Consul service discovery sidecar to finish setting up before attempting
        // to connect.
        //
        // The Rust integration tests use a Consul Connect sidecar to for connecting to upstream
        // services via a port bound to the loopback interface. Trying to use the sidecar at this
        // point will likely result in "connection refused" errors.
        //
        // While at this point we could connect directly to the web-ui service at
        // http://web-ui.service.dc1.consul:1234 without this annoying wait, doing so won't work
        // for authenticated APIs because the web-ui service sets the session cookie as
        // Secure-Only, which means it won't be sent to the host - sending a Secure-Only cookie
        // over a non-TLS connection only works when the endpoint address is 127.0.0.1. When/if the
        // ingres service serves the web-ui behind TLS then we can skip this wait and connect right
        // away via https://web-ui.service.dc1.consul:1234. This would have the additional benefit
        // of including the ingress-service in this integration test.
        //
        // TODO: remove this when we either a) can rely to service discovery to finish setting up,
        // or b) connect via the ingress-service.
        TestApp::wait_for_service_discovery();

        Ok(Self {
            endpoint_url,
            client,
            test_user,
        })
    }

    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        let endpoint_url = self.endpoint_url.as_str();
        self.client.post(format!("{endpoint_url}{path}"))
    }

    /// Login with test user credentials. The web client will save session cookies and use
    /// them in future requests, allowing the client to use authenticated APIs.
    pub async fn login_with_test_user(&self) -> eyre::Result<reqwest::Response> {
        let response = self
            .post("api/auth/sign_in_with_password")
            .json(&serde_json::json!({
                "username": self.test_user.username,
                "password": self.test_user.password,
            }))
            .send()
            .await?;

        match &response.status() {
            &actix_web::http::StatusCode::OK => Ok(response),
            _ => Err(eyre::eyre!("unable to log in with test user: {response:?}")),
        }
    }

    fn wait_for_service_discovery() {
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
}
