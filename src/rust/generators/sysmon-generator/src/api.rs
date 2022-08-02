use rust_proto::graplinc::grapl::api::plugin_sdk::generators::v1beta1::{
    server::GeneratorApi,
    GeneratedGraph,
    RunGeneratorRequest,
    RunGeneratorResponse,
};
use sysmon_parser::SysmonEvent;

use crate::{
    error::SysmonGeneratorError,
    models,
};

pub struct SysmonGenerator {}

#[async_trait::async_trait]
impl GeneratorApi for SysmonGenerator {
    type Error = SysmonGeneratorError;

    #[tracing::instrument(skip(self, request), err)]
    async fn run_generator(
        &self,
        request: RunGeneratorRequest,
    ) -> Result<RunGeneratorResponse, Self::Error> {
        let input_utf8 = std::str::from_utf8(&request.data)?;
        let events: Vec<_> = sysmon_parser::parse_events(input_utf8).collect();
        let sysmon_event: SysmonEvent = expect_one_event(events)?;

        match models::generate_graph_from_event(&sysmon_event)? {
            Some(graph_description) => Ok(RunGeneratorResponse {
                generated_graph: GeneratedGraph { graph_description },
            }),
            None => {
                // We do not expect to handle all Sysmon event types.
                // So we'd just return an empty Graph Description.
                Ok(RunGeneratorResponse::default())
            }
        }
    }
}

pub fn expect_one_event(
    events: Vec<Result<sysmon_parser::SysmonEvent, sysmon_parser::Error>>,
) -> Result<sysmon_parser::SysmonEvent, sysmon_parser::Error> {
    if events.len() != 1 {
        tracing::warn!(
            message =
                "sysmon-generator expects inputs of exactly 1 event - dropping all other events!",
            found_events = events.len()
        );
    }
    // Take first, or EventNotFound
    events
        .into_iter()
        .next()
        .unwrap_or_else(|| Err(sysmon_parser::Error::SysmonEventNotFound))
}
