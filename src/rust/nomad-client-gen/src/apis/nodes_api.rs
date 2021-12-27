/*
 * Nomad
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 1.1.4
 * Contact: support@hashicorp.com
 * Generated by: https://openapi-generator.tech
 */

use std::rc::Rc;
use std::borrow::Borrow;
use std::pin::Pin;
#[allow(unused_imports)]
use std::option::Option;

use hyper;
use futures::Future;

use super::{Error, configuration};
use super::request as __internal_request;

pub struct NodesApiClient<C: hyper::client::connect::Connect>
    where C: Clone + std::marker::Send + Sync + 'static {
    configuration: Rc<configuration::Configuration<C>>,
}

impl<C: hyper::client::connect::Connect> NodesApiClient<C>
    where C: Clone + std::marker::Send + Sync {
    pub fn new(configuration: Rc<configuration::Configuration<C>>) -> NodesApiClient<C> {
        NodesApiClient {
            configuration,
        }
    }
}

pub trait NodesApi {
    fn get_node(&self, node_id: &str, region: Option<&str>, namespace: Option<&str>, index: Option<i32>, wait: Option<&str>, stale: Option<&str>, prefix: Option<&str>, x_nomad_token: Option<&str>, per_page: Option<i32>, next_token: Option<&str>) -> Pin<Box<dyn Future<Output = Result<crate::models::Node, Error>>>>;
    fn get_node_allocations(&self, node_id: &str, region: Option<&str>, namespace: Option<&str>, index: Option<i32>, wait: Option<&str>, stale: Option<&str>, prefix: Option<&str>, x_nomad_token: Option<&str>, per_page: Option<i32>, next_token: Option<&str>) -> Pin<Box<dyn Future<Output = Result<Vec<crate::models::AllocationListStub>, Error>>>>;
    fn get_nodes(&self, region: Option<&str>, namespace: Option<&str>, index: Option<i32>, wait: Option<&str>, stale: Option<&str>, prefix: Option<&str>, x_nomad_token: Option<&str>, per_page: Option<i32>, next_token: Option<&str>, resources: Option<bool>) -> Pin<Box<dyn Future<Output = Result<Vec<crate::models::NodeListStub>, Error>>>>;
    fn update_node_drain(&self, node_id: &str, node_update_drain_request: crate::models::NodeUpdateDrainRequest, region: Option<&str>, namespace: Option<&str>, index: Option<i32>, wait: Option<&str>, stale: Option<&str>, prefix: Option<&str>, x_nomad_token: Option<&str>, per_page: Option<i32>, next_token: Option<&str>) -> Pin<Box<dyn Future<Output = Result<crate::models::NodeDrainUpdateResponse, Error>>>>;
    fn update_node_eligibility(&self, node_id: &str, node_update_eligibility_request: crate::models::NodeUpdateEligibilityRequest, region: Option<&str>, namespace: Option<&str>, index: Option<i32>, wait: Option<&str>, stale: Option<&str>, prefix: Option<&str>, x_nomad_token: Option<&str>, per_page: Option<i32>, next_token: Option<&str>) -> Pin<Box<dyn Future<Output = Result<crate::models::NodeEligibilityUpdateResponse, Error>>>>;
    fn update_node_purge(&self, node_id: &str, region: Option<&str>, namespace: Option<&str>, index: Option<i32>, wait: Option<&str>, stale: Option<&str>, prefix: Option<&str>, x_nomad_token: Option<&str>, per_page: Option<i32>, next_token: Option<&str>) -> Pin<Box<dyn Future<Output = Result<crate::models::NodePurgeResponse, Error>>>>;
}

impl<C: hyper::client::connect::Connect>NodesApi for NodesApiClient<C>
    where C: Clone + std::marker::Send + Sync {
    #[allow(unused_mut)]
    fn get_node(&self, node_id: &str, region: Option<&str>, namespace: Option<&str>, index: Option<i32>, wait: Option<&str>, stale: Option<&str>, prefix: Option<&str>, x_nomad_token: Option<&str>, per_page: Option<i32>, next_token: Option<&str>) -> Pin<Box<dyn Future<Output = Result<crate::models::Node, Error>>>> {
        let mut req = __internal_request::Request::new(hyper::Method::GET, "/node/{nodeId}".to_string())
            .with_auth(__internal_request::Auth::ApiKey(__internal_request::ApiKey{
                in_header: true,
                in_query: false,
                param_name: "X-Nomad-Token".to_owned(),
            }))
        ;
        if let Some(ref s) = region {
            let query_value = s.to_string();
            req = req.with_query_param("region".to_string(), query_value);
        }
        if let Some(ref s) = namespace {
            let query_value = s.to_string();
            req = req.with_query_param("namespace".to_string(), query_value);
        }
        if let Some(ref s) = wait {
            let query_value = s.to_string();
            req = req.with_query_param("wait".to_string(), query_value);
        }
        if let Some(ref s) = stale {
            let query_value = s.to_string();
            req = req.with_query_param("stale".to_string(), query_value);
        }
        if let Some(ref s) = prefix {
            let query_value = s.to_string();
            req = req.with_query_param("prefix".to_string(), query_value);
        }
        if let Some(ref s) = per_page {
            let query_value = s.to_string();
            req = req.with_query_param("per_page".to_string(), query_value);
        }
        if let Some(ref s) = next_token {
            let query_value = s.to_string();
            req = req.with_query_param("next_token".to_string(), query_value);
        }
        req = req.with_path_param("nodeId".to_string(), node_id.to_string());
        if let Some(param_value) = index {
            req = req.with_header_param("index".to_string(), param_value.to_string());
        }
        if let Some(param_value) = x_nomad_token {
            req = req.with_header_param("X-Nomad-Token".to_string(), param_value.to_string());
        }

        req.execute(self.configuration.borrow())
    }

    #[allow(unused_mut)]
    fn get_node_allocations(&self, node_id: &str, region: Option<&str>, namespace: Option<&str>, index: Option<i32>, wait: Option<&str>, stale: Option<&str>, prefix: Option<&str>, x_nomad_token: Option<&str>, per_page: Option<i32>, next_token: Option<&str>) -> Pin<Box<dyn Future<Output = Result<Vec<crate::models::AllocationListStub>, Error>>>> {
        let mut req = __internal_request::Request::new(hyper::Method::GET, "/node/{nodeId}/allocations".to_string())
            .with_auth(__internal_request::Auth::ApiKey(__internal_request::ApiKey{
                in_header: true,
                in_query: false,
                param_name: "X-Nomad-Token".to_owned(),
            }))
        ;
        if let Some(ref s) = region {
            let query_value = s.to_string();
            req = req.with_query_param("region".to_string(), query_value);
        }
        if let Some(ref s) = namespace {
            let query_value = s.to_string();
            req = req.with_query_param("namespace".to_string(), query_value);
        }
        if let Some(ref s) = wait {
            let query_value = s.to_string();
            req = req.with_query_param("wait".to_string(), query_value);
        }
        if let Some(ref s) = stale {
            let query_value = s.to_string();
            req = req.with_query_param("stale".to_string(), query_value);
        }
        if let Some(ref s) = prefix {
            let query_value = s.to_string();
            req = req.with_query_param("prefix".to_string(), query_value);
        }
        if let Some(ref s) = per_page {
            let query_value = s.to_string();
            req = req.with_query_param("per_page".to_string(), query_value);
        }
        if let Some(ref s) = next_token {
            let query_value = s.to_string();
            req = req.with_query_param("next_token".to_string(), query_value);
        }
        req = req.with_path_param("nodeId".to_string(), node_id.to_string());
        if let Some(param_value) = index {
            req = req.with_header_param("index".to_string(), param_value.to_string());
        }
        if let Some(param_value) = x_nomad_token {
            req = req.with_header_param("X-Nomad-Token".to_string(), param_value.to_string());
        }

        req.execute(self.configuration.borrow())
    }

    #[allow(unused_mut)]
    fn get_nodes(&self, region: Option<&str>, namespace: Option<&str>, index: Option<i32>, wait: Option<&str>, stale: Option<&str>, prefix: Option<&str>, x_nomad_token: Option<&str>, per_page: Option<i32>, next_token: Option<&str>, resources: Option<bool>) -> Pin<Box<dyn Future<Output = Result<Vec<crate::models::NodeListStub>, Error>>>> {
        let mut req = __internal_request::Request::new(hyper::Method::GET, "/nodes".to_string())
            .with_auth(__internal_request::Auth::ApiKey(__internal_request::ApiKey{
                in_header: true,
                in_query: false,
                param_name: "X-Nomad-Token".to_owned(),
            }))
        ;
        if let Some(ref s) = region {
            let query_value = s.to_string();
            req = req.with_query_param("region".to_string(), query_value);
        }
        if let Some(ref s) = namespace {
            let query_value = s.to_string();
            req = req.with_query_param("namespace".to_string(), query_value);
        }
        if let Some(ref s) = wait {
            let query_value = s.to_string();
            req = req.with_query_param("wait".to_string(), query_value);
        }
        if let Some(ref s) = stale {
            let query_value = s.to_string();
            req = req.with_query_param("stale".to_string(), query_value);
        }
        if let Some(ref s) = prefix {
            let query_value = s.to_string();
            req = req.with_query_param("prefix".to_string(), query_value);
        }
        if let Some(ref s) = per_page {
            let query_value = s.to_string();
            req = req.with_query_param("per_page".to_string(), query_value);
        }
        if let Some(ref s) = next_token {
            let query_value = s.to_string();
            req = req.with_query_param("next_token".to_string(), query_value);
        }
        if let Some(ref s) = resources {
            let query_value = s.to_string();
            req = req.with_query_param("resources".to_string(), query_value);
        }
        if let Some(param_value) = index {
            req = req.with_header_param("index".to_string(), param_value.to_string());
        }
        if let Some(param_value) = x_nomad_token {
            req = req.with_header_param("X-Nomad-Token".to_string(), param_value.to_string());
        }

        req.execute(self.configuration.borrow())
    }

    #[allow(unused_mut)]
    fn update_node_drain(&self, node_id: &str, node_update_drain_request: crate::models::NodeUpdateDrainRequest, region: Option<&str>, namespace: Option<&str>, index: Option<i32>, wait: Option<&str>, stale: Option<&str>, prefix: Option<&str>, x_nomad_token: Option<&str>, per_page: Option<i32>, next_token: Option<&str>) -> Pin<Box<dyn Future<Output = Result<crate::models::NodeDrainUpdateResponse, Error>>>> {
        let mut req = __internal_request::Request::new(hyper::Method::POST, "/node/{nodeId}/drain".to_string())
            .with_auth(__internal_request::Auth::ApiKey(__internal_request::ApiKey{
                in_header: true,
                in_query: false,
                param_name: "X-Nomad-Token".to_owned(),
            }))
        ;
        if let Some(ref s) = region {
            let query_value = s.to_string();
            req = req.with_query_param("region".to_string(), query_value);
        }
        if let Some(ref s) = namespace {
            let query_value = s.to_string();
            req = req.with_query_param("namespace".to_string(), query_value);
        }
        if let Some(ref s) = wait {
            let query_value = s.to_string();
            req = req.with_query_param("wait".to_string(), query_value);
        }
        if let Some(ref s) = stale {
            let query_value = s.to_string();
            req = req.with_query_param("stale".to_string(), query_value);
        }
        if let Some(ref s) = prefix {
            let query_value = s.to_string();
            req = req.with_query_param("prefix".to_string(), query_value);
        }
        if let Some(ref s) = per_page {
            let query_value = s.to_string();
            req = req.with_query_param("per_page".to_string(), query_value);
        }
        if let Some(ref s) = next_token {
            let query_value = s.to_string();
            req = req.with_query_param("next_token".to_string(), query_value);
        }
        req = req.with_path_param("nodeId".to_string(), node_id.to_string());
        if let Some(param_value) = index {
            req = req.with_header_param("index".to_string(), param_value.to_string());
        }
        if let Some(param_value) = x_nomad_token {
            req = req.with_header_param("X-Nomad-Token".to_string(), param_value.to_string());
        }
        req = req.with_body_param(node_update_drain_request);

        req.execute(self.configuration.borrow())
    }

    #[allow(unused_mut)]
    fn update_node_eligibility(&self, node_id: &str, node_update_eligibility_request: crate::models::NodeUpdateEligibilityRequest, region: Option<&str>, namespace: Option<&str>, index: Option<i32>, wait: Option<&str>, stale: Option<&str>, prefix: Option<&str>, x_nomad_token: Option<&str>, per_page: Option<i32>, next_token: Option<&str>) -> Pin<Box<dyn Future<Output = Result<crate::models::NodeEligibilityUpdateResponse, Error>>>> {
        let mut req = __internal_request::Request::new(hyper::Method::POST, "/node/{nodeId}/eligibility".to_string())
            .with_auth(__internal_request::Auth::ApiKey(__internal_request::ApiKey{
                in_header: true,
                in_query: false,
                param_name: "X-Nomad-Token".to_owned(),
            }))
        ;
        if let Some(ref s) = region {
            let query_value = s.to_string();
            req = req.with_query_param("region".to_string(), query_value);
        }
        if let Some(ref s) = namespace {
            let query_value = s.to_string();
            req = req.with_query_param("namespace".to_string(), query_value);
        }
        if let Some(ref s) = wait {
            let query_value = s.to_string();
            req = req.with_query_param("wait".to_string(), query_value);
        }
        if let Some(ref s) = stale {
            let query_value = s.to_string();
            req = req.with_query_param("stale".to_string(), query_value);
        }
        if let Some(ref s) = prefix {
            let query_value = s.to_string();
            req = req.with_query_param("prefix".to_string(), query_value);
        }
        if let Some(ref s) = per_page {
            let query_value = s.to_string();
            req = req.with_query_param("per_page".to_string(), query_value);
        }
        if let Some(ref s) = next_token {
            let query_value = s.to_string();
            req = req.with_query_param("next_token".to_string(), query_value);
        }
        req = req.with_path_param("nodeId".to_string(), node_id.to_string());
        if let Some(param_value) = index {
            req = req.with_header_param("index".to_string(), param_value.to_string());
        }
        if let Some(param_value) = x_nomad_token {
            req = req.with_header_param("X-Nomad-Token".to_string(), param_value.to_string());
        }
        req = req.with_body_param(node_update_eligibility_request);

        req.execute(self.configuration.borrow())
    }

    #[allow(unused_mut)]
    fn update_node_purge(&self, node_id: &str, region: Option<&str>, namespace: Option<&str>, index: Option<i32>, wait: Option<&str>, stale: Option<&str>, prefix: Option<&str>, x_nomad_token: Option<&str>, per_page: Option<i32>, next_token: Option<&str>) -> Pin<Box<dyn Future<Output = Result<crate::models::NodePurgeResponse, Error>>>> {
        let mut req = __internal_request::Request::new(hyper::Method::POST, "/node/{nodeId}/purge".to_string())
            .with_auth(__internal_request::Auth::ApiKey(__internal_request::ApiKey{
                in_header: true,
                in_query: false,
                param_name: "X-Nomad-Token".to_owned(),
            }))
        ;
        if let Some(ref s) = region {
            let query_value = s.to_string();
            req = req.with_query_param("region".to_string(), query_value);
        }
        if let Some(ref s) = namespace {
            let query_value = s.to_string();
            req = req.with_query_param("namespace".to_string(), query_value);
        }
        if let Some(ref s) = wait {
            let query_value = s.to_string();
            req = req.with_query_param("wait".to_string(), query_value);
        }
        if let Some(ref s) = stale {
            let query_value = s.to_string();
            req = req.with_query_param("stale".to_string(), query_value);
        }
        if let Some(ref s) = prefix {
            let query_value = s.to_string();
            req = req.with_query_param("prefix".to_string(), query_value);
        }
        if let Some(ref s) = per_page {
            let query_value = s.to_string();
            req = req.with_query_param("per_page".to_string(), query_value);
        }
        if let Some(ref s) = next_token {
            let query_value = s.to_string();
            req = req.with_query_param("next_token".to_string(), query_value);
        }
        req = req.with_path_param("nodeId".to_string(), node_id.to_string());
        if let Some(param_value) = index {
            req = req.with_header_param("index".to_string(), param_value.to_string());
        }
        if let Some(param_value) = x_nomad_token {
            req = req.with_header_param("X-Nomad-Token".to_string(), param_value.to_string());
        }

        req.execute(self.configuration.borrow())
    }

}
