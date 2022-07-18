use grapl_tracing::setup_tracing;
use rust_proto::graplinc::grapl::{
    api::graph::v1beta1::GraphDescription,
    pipeline::{
        v1beta1::RawLog,
        v1beta2::Envelope,
    },
};
use thiserror::Error;

mod parsers;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum OsqueryGeneratorError {
}

const SERVICE_NAME: &'static str = "pipeline-ingress";

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = setup_tracing(SERVICE_NAME);

    // TODO: actually do something here
    Ok(())
}

// TODO: when we have a plugin SDK, hook this binary into it here
#[allow(dead_code)]
async fn event_handler(
    _event: Envelope<RawLog>,
) -> Result<Envelope<GraphDescription>, OsqueryGeneratorError> {
    todo!();
}
