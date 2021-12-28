use http;
use hyper;
use serde_json;

#[derive(Debug)]
pub enum Error {
    Api(ApiError),
    Header(hyper::http::header::InvalidHeaderValue),
    Http(http::Error),
    Hyper(hyper::Error),
    Serde(serde_json::Error),
    UriError(http::uri::InvalidUri),
}

#[derive(Debug)]
pub struct ApiError {
    pub code: hyper::StatusCode,
    pub body: hyper::body::Body,
}

impl From<(hyper::StatusCode, hyper::body::Body)> for Error {
    fn from(e: (hyper::StatusCode, hyper::body::Body)) -> Self {
        Error::Api(ApiError {
            code: e.0,
            body: e.1,
        })
    }
}

impl From<http::Error> for Error {
    fn from(e: http::Error) -> Self {
        return Error::Http(e);
    }
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Self {
        return Error::Hyper(e);
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        return Error::Serde(e);
    }
}

mod request;

mod acl_api;
pub use self::acl_api::{
    ACLApi,
    ACLApiClient,
};
mod allocations_api;
pub use self::allocations_api::{
    AllocationsApi,
    AllocationsApiClient,
};
mod deployments_api;
pub use self::deployments_api::{
    DeploymentsApi,
    DeploymentsApiClient,
};
mod enterprise_api;
pub use self::enterprise_api::{
    EnterpriseApi,
    EnterpriseApiClient,
};
mod evaluations_api;
pub use self::evaluations_api::{
    EvaluationsApi,
    EvaluationsApiClient,
};
mod jobs_api;
pub use self::jobs_api::{
    JobsApi,
    JobsApiClient,
};
mod metrics_api;
pub use self::metrics_api::{
    MetricsApi,
    MetricsApiClient,
};
mod namespaces_api;
pub use self::namespaces_api::{
    NamespacesApi,
    NamespacesApiClient,
};
mod nodes_api;
pub use self::nodes_api::{
    NodesApi,
    NodesApiClient,
};
mod plugins_api;
pub use self::plugins_api::{
    PluginsApi,
    PluginsApiClient,
};
mod regions_api;
pub use self::regions_api::{
    RegionsApi,
    RegionsApiClient,
};
mod scaling_api;
pub use self::scaling_api::{
    ScalingApi,
    ScalingApiClient,
};
mod search_api;
pub use self::search_api::{
    SearchApi,
    SearchApiClient,
};
mod system_api;
pub use self::system_api::{
    SystemApi,
    SystemApiClient,
};
mod volumes_api;
pub use self::volumes_api::{
    VolumesApi,
    VolumesApiClient,
};

pub mod client;
pub mod configuration;
