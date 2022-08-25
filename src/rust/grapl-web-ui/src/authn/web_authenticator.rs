use std::ops::Add;

use argon2::{
    Algorithm,
    Argon2,
    Params,
    PasswordHash,
    PasswordVerifier,
    Version,
};
use chrono::{
    TimeZone,
    Utc,
};
use rand::Rng;
use secrecy::ExposeSecret;

use super::{
    authenticated_user::AuthenticatedUser,
    dynamodb_client::AuthDynamoClient,
    error::{
        AuthenticationError,
        Result,
    },
    Secret,
};
use crate::config::{
    SESSION_EXPIRATION_TIMEOUT_DAYS,
    SESSION_TOKEN_LENGTH,
};

type SessionToken = Secret<String>;

pub(crate) struct WebAuthenticator {
    db_client: AuthDynamoClient,
    google_web_client: jsonwebtoken_google::Parser,
}

impl WebAuthenticator {
    pub(crate) fn new(
        db_client: AuthDynamoClient,
        google_web_client: jsonwebtoken_google::Parser,
    ) -> Self {
        Self {
            db_client,
            google_web_client,
        }
    }

    #[tracing::instrument(skip(self, password))]
    pub(crate) async fn sign_in_with_pw(
        &self,
        username: String,
        password: Secret<String>,
    ) -> Result<SessionToken> {
        let user_database_row = self
            .db_client
            .get_user_row(username.clone())
            .await?
            .ok_or_else(|| AuthenticationError::UserNotFound(username.clone()))?;

        let password_hasher = Argon2::new(
            Algorithm::Argon2i,
            Version::V0x13,
            Params::new(102400, 2, 8, None)?,
        );

        let hash_from_database = user_database_row.get_password_hash();
        let hash_from_database = PasswordHash::new(hash_from_database.expose_secret())
            .map_err(|source| AuthenticationError::DeserializePasswordFromDatabase { source })?;

        password_hasher
            .verify_password(password.expose_secret().as_bytes(), &hash_from_database)
            .map_err(|source| AuthenticationError::InvalidPassword { username, source })?;

        // password verification has succeeded

        self.create_web_session(user_database_row.get_username())
            .await
    }

    #[tracing::instrument(skip(self, token))]
    pub(crate) async fn sign_in_with_google(&self, token: Secret<String>) -> Result<SessionToken> {
        #[derive(Debug, serde::Deserialize)]
        //TODO(inickles): validate this needs to be dead_code
        #[allow(dead_code)]
        struct TokenClaims {
            email: String,
            aud: String,
            iss: String,
            exp: u64,
        }

        let token_claims = self
            .google_web_client
            .parse::<TokenClaims>(token.expose_secret())
            .await?;
        let google_email = token_claims.email;

        tracing::info!(
            message = "Sign In With Google token verification successful",
            username = google_email
        );

        // Look up username in the user auth database
        let user_row = self
            .db_client
            .get_user_row(google_email.clone())
            .await?
            .ok_or(AuthenticationError::UserNotFound(google_email))?;

        self.create_web_session(user_row.get_username()).await
    }

    pub(crate) async fn validate_session_token(
        &self,
        session: Secret<String>,
    ) -> Result<AuthenticatedUser> {
        let session_row = self
            .db_client
            .get_web_session_row(session)
            .await?
            .ok_or(AuthenticationError::SessionTokenNotFound)?;

        let username = session_row.get_username();
        let expiration = Utc.timestamp(session_row.get_expiration(), 0);

        if Utc::now().ge(&expiration) {
            return Err(AuthenticationError::SessionExpired);
        }

        let user_row = self
            .db_client
            .get_user_row(username.to_string())
            .await?
            .ok_or(AuthenticationError::UserNotFound(username.to_string()))?;

        let organization_id =
            uuid::Uuid::parse_str(user_row.get_organization_id()).map_err(|source| {
                AuthenticationError::ParseOrgId {
                    input: user_row.get_organization_id().to_string(),
                    source,
                }
            })?;

        let authenticated_user = AuthenticatedUser::new(
            user_row.get_username().to_owned(),
            user_row.get_role().to_owned(),
            organization_id,
        );

        Ok(authenticated_user)
    }

    async fn create_web_session(&self, username: &str) -> Result<Secret<String>> {
        let token_value: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(SESSION_TOKEN_LENGTH)
            .map(char::from)
            .collect();

        let expiration = chrono::Utc::now()
            .add(chrono::Duration::days(SESSION_EXPIRATION_TIMEOUT_DAYS))
            .timestamp();

        self.db_client
            .store_web_session(username.to_owned(), token_value.clone(), expiration)
            .await?;

        Ok(token_value.into())
    }
}
