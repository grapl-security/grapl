use grapl_config::ServiceEnv;
use log::info;
use rusoto_core::{HttpClient, Region};
use rusoto_dynamodb::DynamoDbClient;
use rusoto_s3::S3Client;
use rusoto_sqs::SqsClient;

pub trait AwsClientFactory: Sync + Send {
    fn get_sqs_client(&self) -> SqsClient;
    fn get_s3_client(&self) -> S3Client;
    fn get_dynamodb_client(&self) -> DynamoDbClient;
}

pub fn new_aws_client_factory(env: &ServiceEnv) -> Box<dyn AwsClientFactory> {
    match env.is_local {
        true => Box::new(LocalAwsClientFactory {}),
        false => Box::new(ProdAwsClientFactory {
            region: env.get_region(),
        }),
    }
}

pub struct ProdAwsClientFactory {
    region: Region,
}

impl AwsClientFactory for ProdAwsClientFactory {
    fn get_sqs_client(&self) -> SqsClient {
        SqsClient::new(self.region.clone())
    }
    fn get_s3_client(&self) -> S3Client {
        S3Client::new(self.region.clone())
    }
    fn get_dynamodb_client(&self) -> DynamoDbClient {
        DynamoDbClient::new(self.region.clone())
    }
}

pub struct LocalAwsClientFactory {}

impl AwsClientFactory for LocalAwsClientFactory {
    fn get_sqs_client(&self) -> SqsClient {
        info!("Connecting to local us-east-1 http://sqs.us-east-1.amazonaws.com:9324");

        SqsClient::new_with(
            HttpClient::new().expect("failed to create request dispatcher"),
            rusoto_credential::StaticProvider::new_minimal(
                "dummy_sqs".to_owned(),
                "dummy_sqs".to_owned(),
            ),
            Region::Custom {
                name: "us-east-1".to_string(),
                endpoint: "http://sqs.us-east-1.amazonaws.com:9324".to_string(),
            },
        )
    }

    fn get_s3_client(&self) -> S3Client {
        info!("Connecting to local http://s3:9000");
        S3Client::new_with(
            HttpClient::new().expect("failed to create request dispatcher"),
            rusoto_credential::StaticProvider::new_minimal(
                "minioadmin".to_owned(),
                "minioadmin".to_owned(),
            ),
            Region::Custom {
                name: "locals3".to_string(),
                endpoint: "http://s3:9000".to_string(),
            },
        )
    }

    fn get_dynamodb_client(&self) -> DynamoDbClient {
        info!("Connecting to local http://dynamodb:8000");
        DynamoDbClient::new_with(
            HttpClient::new().expect("failed to create request dispatcher"),
            rusoto_credential::StaticProvider::new_minimal(
                "dummy_cred_aws_access_key_id".to_owned(),
                "dummy_cred_aws_secret_access_key".to_owned(),
            ),
            Region::Custom {
                name: "us-west-2".to_string(),
                endpoint: "http://dynamodb:8000".to_string(),
            },
        )
    }
}
