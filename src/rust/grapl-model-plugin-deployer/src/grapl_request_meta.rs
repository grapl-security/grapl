use crate::grapl_model_plugin_deployer_proto::GraplRequestMeta;

impl GraplRequestMeta {
    pub fn get_client_name(&self) -> &str {
        self.client_name.as_ref()
    }

    pub fn get_request_id(&self) -> &str {
        self.request_id.as_ref()
    }
}
