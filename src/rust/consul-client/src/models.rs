#[derive(Debug)]
pub struct CheckHealthResponse(pub Vec<CheckHealthResponseElem>);

impl CheckHealthResponse {
    /// Return whether all service healthchecks in a CheckHealthResponse
    /// have state "passing"
    pub fn all_passing(&self) -> bool {
        if self.0.is_empty() {
            // no such service found - it may still be booting up
            return false;
        }

        if self.0.iter().all(|elem| elem.status == "passing") {
            return true;
        }

        // at least one such service is critical
        false
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct CheckHealthResponseElem {
    #[serde(alias = "Status")]
    pub status: String,
    #[serde(alias = "ServiceName")]
    pub service_name: String,
}
