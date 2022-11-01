//

#[derive(thiserror::Error, Debug)]
pub enum ConsulClientError {
    #[error("consul check health error: '{0}'")]
    CheckHealthError(reqwest::Error),
    #[error("reqwest serde error: '{0}'")]
    ReqwestSerdeError(reqwest::Error),
}

#[derive(Debug)]
pub struct CheckHealthResponse(Vec<CheckHealthResponseElem>);

#[derive(serde::Deserialize, Debug)]
pub struct CheckHealthResponseElem {}

pub async fn check_health(
    service_name: impl Into<String>,
) -> Result<CheckHealthResponse, ConsulClientError> {
    let service_name = service_name.into();
    let url = format!("http://consul.service.consul:8500/v1/health/checks/{service_name}");
    let response = reqwest::get(url)
        .await
        .map_err(ConsulClientError::CheckHealthError)?;
    let responses = response
        .json::<Vec<CheckHealthResponseElem>>()
        .await
        .map_err(ConsulClientError::ReqwestSerdeError)?;
    Ok(CheckHealthResponse(responses))
}
