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
        let sysmon_event = SysmonEvent::from_str(std::str::from_utf8(&request.data)?)?;

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
