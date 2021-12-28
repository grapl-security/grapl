/*
 * Nomad
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 1.1.4
 * Contact: support@hashicorp.com
 * Generated by: https://openapi-generator.tech
 */

#[allow(unused_imports)]
use std::option::Option;
use std::{
    borrow::Borrow,
    pin::Pin,
    rc::Rc,
};

use futures::Future;
use hyper;

use super::{
    configuration,
    request as __internal_request,
    Error,
};

pub struct MetricsApiClient<C: hyper::client::connect::Connect>
where
    C: Clone + std::marker::Send + Sync + 'static,
{
    configuration: Rc<configuration::Configuration<C>>,
}

impl<C: hyper::client::connect::Connect> MetricsApiClient<C>
where
    C: Clone + std::marker::Send + Sync,
{
    pub fn new(configuration: Rc<configuration::Configuration<C>>) -> MetricsApiClient<C> {
        MetricsApiClient { configuration }
    }
}

pub trait MetricsApi {
    fn get_metrics_summary(
        &self,
        format: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<crate::models::MetricsSummary, Error>>>>;
}

impl<C: hyper::client::connect::Connect> MetricsApi for MetricsApiClient<C>
where
    C: Clone + std::marker::Send + Sync,
{
    #[allow(unused_mut)]
    fn get_metrics_summary(
        &self,
        format: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<crate::models::MetricsSummary, Error>>>> {
        let mut req = __internal_request::Request::new(hyper::Method::GET, "/metrics".to_string())
            .with_auth(__internal_request::Auth::ApiKey(
                __internal_request::ApiKey {
                    in_header: true,
                    in_query: false,
                    param_name: "X-Nomad-Token".to_owned(),
                },
            ));
        if let Some(ref s) = format {
            let query_value = s.to_string();
            req = req.with_query_param("format".to_string(), query_value);
        }

        req.execute(self.configuration.borrow())
    }
}
