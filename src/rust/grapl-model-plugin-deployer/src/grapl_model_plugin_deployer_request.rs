use crate::grapl_model_plugin_deployer_proto::{
    GraplModelPluginDeployerRequest,
    GraplRequestMeta,
};

impl GraplModelPluginDeployerRequest {
    pub fn expect_client_name(&self) -> &str {
        self.get_client_name().expect("client_name")
    }

    pub fn expect_request_id(&self) -> &str {
        self.get_request_id().expect("request_id")
    }

    pub fn get_client_name(&self) -> Option<&str> {
        self.request_meta
            .as_ref()
            .map(GraplRequestMeta::get_client_name)
    }

    pub fn get_request_id(&self) -> Option<&str> {
        self.request_meta
            .as_ref()
            .map(GraplRequestMeta::get_request_id)
    }
}
