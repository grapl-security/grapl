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

        Ok(Self {
            endpoint_url,
            client,
            test_user,
        })
    }

    /// Send a reqwest::RequestBuilder and return immediately unless a 500 error was returned.
    /// In that case, retry the request up to ten times before returning the last error response.
    ///
    /// This is a (hopefully temporary) mitigation around intermittent errors we're getting from
    /// the Consul sidecar in Nomad.
    /// See: https://github.com/grapl-security/issue-tracker/issues/1008
    pub async fn send_with_retries(
        &self,
        request: reqwest::RequestBuilder,
    ) -> eyre::Result<reqwest::Response> {
        let num_retries = 10;
        let mut response = request
            .try_clone()
            .ok_or_else(|| eyre::eyre!("Unable to clone request - perhaps it is a stream?"))?
            .send()
            .await?;

        for _ in 1..num_retries {
            let status_code = response.status().as_u16();

            if status_code >= 500 && status_code <= 599 {
                // We recevied a 500 error, wait a moment before trying the request again
                println!("5xx Error: {:?}", response);

                let one_sec = std::time::Duration::from_secs(1);
                std::thread::sleep(one_sec);

                response = request
                    .try_clone()
                    .ok_or_else(|| {
                        eyre::eyre!("Unable to clone request - perhaps it is a stream?")
                    })?
                    .send()
                    .await?;

                continue;
            } else {
                // Non-500 error, break the retry loop to return it
                break;
            }
        }

        Ok(response)
    }

    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        let endpoint_url = self.endpoint_url.as_str();
        self.client.post(format!("{endpoint_url}{path}"))
    }

    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        let endpoint_url = self.endpoint_url.as_str();
        self.client.get(format!("{endpoint_url}{path}"))
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
}
