use argon2::{
    Argon2,
    PasswordHasher,
};
use rusoto_dynamodb::{
    AttributeValue,
    DynamoDb,
    DynamoDbClient,
    PutItemInput,
};
use uuid::Uuid;

pub struct TestUser {
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn new() -> Self {
        Self {
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }

    pub async fn store(
        &self,
        user_table_name: &str,
        db_client: &DynamoDbClient,
    ) -> eyre::Result<()> {
        let password_salt = argon2::password_hash::SaltString::generate(&mut rand::thread_rng());
        // Ensure these match what's used in production
        // TODO(inickles): consider just making the hasher public
        let password_hash = Argon2::new(
            argon2::Algorithm::Argon2i,
            argon2::Version::V0x13,
            argon2::Params::new(102400, 2, 8, None)?,
        )
        .hash_password(self.password.as_bytes(), &password_salt)?
        .to_string();

        // Ensure this matches grapl_web_ui::auth::dynamodb_client::UserRow
        let user_entry = hmap::hmap! {
            "username".to_owned() => AttributeValue {
                s: Some(self.username.to_owned()),
                ..Default::default()
            },
            "grapl_role".to_owned() => AttributeValue {
                s: Some("user".to_owned()),
                ..Default::default()
            },
            "password_hash".to_owned() => AttributeValue {
                s: Some(password_hash),
                ..Default::default()
            },
            "organization_id".to_owned() => AttributeValue {
                s: Some("test".to_owned()),
                ..Default::default()
            }
        };

        let new_user_input = PutItemInput {
            item: user_entry,
            table_name: user_table_name.to_owned(),
            ..Default::default()
        };

        db_client.put_item(new_user_input).await?;

        Ok(())
    }
}
