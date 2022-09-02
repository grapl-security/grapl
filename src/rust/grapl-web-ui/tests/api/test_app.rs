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
