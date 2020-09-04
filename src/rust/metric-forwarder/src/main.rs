#![type_length_limit = "1214269"]
// Our types are simply too powerful

mod deser_logs_data;
mod error;

use aws_lambda_events::event::cloudwatch_logs::{CloudwatchLogsData, CloudwatchLogsEvent};
use lambda_runtime::error::HandlerError;
use lambda_runtime::lambda;
use lambda_runtime::Context;
use log::info;

use tokio::runtime::Runtime;

// not yet, gotta solve openssl issue
// use rusoto_cloudwatch::{CloudWatch, CloudWatchClient};

fn handler(event: CloudwatchLogsEvent, ctx: Context) -> Result<(), HandlerError> {
    info!("Handling event");

    let logs = deser_logs_data::aws_event_to_cloudwatch_logs_data(event);
    match logs {
        Ok(logs) => {
            // Now we have the actual logs.
            // Parse the statsd format,
            // then forward them to CloudWatch
            Ok(())
        }
        Err(e) => Err(HandlerError::from(e.to_string().as_str())),
    }
}

/*
// TODO wimax Sep'20 - need to figure out how to do a local cloudwatch...
fn init_cloudwatch_client() -> CloudwatchClient {
    info!("Connecting to local http://s3:9000");
    CloudWatchClient::new_with(
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
*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    grapl_config::init_grapl_log!();

    let is_local = std::env::var("IS_LOCAL").is_ok();

    if is_local {
        info!("Running locally");

    /*
    loop {
        if let Err(e) = local_handler().await {
            error!("local_handler: {}", e);
        };
    }
     */
    } else {
        info!("Running in AWS");
        lambda!(handler);
    }

    Ok(())
}
