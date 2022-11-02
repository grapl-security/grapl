#[derive(Debug)]
pub struct CheckHealthResponse(pub Vec<CheckHealthResponseElem>);

impl CheckHealthResponse {
    pub fn all_passing(&self) -> bool {
        if self.0.len() == 0 {
            tracing::info!("no service found");
            // no such service found
            return false;
        }

        if self.0.iter().all(|elem| elem.status == "passing") {
            tracing::info!("all pass");
            return true;
        }

        tracing::info!("other");
        return false;
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct CheckHealthResponseElem {
    #[serde(alias = "Status")]
    pub status: String,
    #[serde(alias = "ServiceName")]
    pub service_name: String,
}
