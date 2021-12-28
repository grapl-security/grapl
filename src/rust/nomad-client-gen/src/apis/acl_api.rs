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

pub struct ACLApiClient<C: hyper::client::connect::Connect>
where
    C: Clone + std::marker::Send + Sync + 'static,
{
    configuration: Rc<configuration::Configuration<C>>,
}

impl<C: hyper::client::connect::Connect> ACLApiClient<C>
where
    C: Clone + std::marker::Send + Sync,
{
    pub fn new(configuration: Rc<configuration::Configuration<C>>) -> ACLApiClient<C> {
        ACLApiClient { configuration }
    }
}

pub trait ACLApi {
    fn delete_acl_policy(
        &self,
        policy_name: &str,
        region: Option<&str>,
        namespace: Option<&str>,
        x_nomad_token: Option<&str>,
        idempotency_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>>>>;
    fn delete_acl_token(
        &self,
        token_accessor: &str,
        region: Option<&str>,
        namespace: Option<&str>,
        x_nomad_token: Option<&str>,
        idempotency_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>>>>;
    fn get_acl_policies(
        &self,
        region: Option<&str>,
        namespace: Option<&str>,
        index: Option<i32>,
        wait: Option<&str>,
        stale: Option<&str>,
        prefix: Option<&str>,
        x_nomad_token: Option<&str>,
        per_page: Option<i32>,
        next_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<crate::models::AclPolicyListStub>, Error>>>>;
    fn get_acl_policy(
        &self,
        policy_name: &str,
        region: Option<&str>,
        namespace: Option<&str>,
        index: Option<i32>,
        wait: Option<&str>,
        stale: Option<&str>,
        prefix: Option<&str>,
        x_nomad_token: Option<&str>,
        per_page: Option<i32>,
        next_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<crate::models::AclPolicy, Error>>>>;
    fn get_acl_token(
        &self,
        token_accessor: &str,
        region: Option<&str>,
        namespace: Option<&str>,
        index: Option<i32>,
        wait: Option<&str>,
        stale: Option<&str>,
        prefix: Option<&str>,
        x_nomad_token: Option<&str>,
        per_page: Option<i32>,
        next_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<crate::models::AclToken, Error>>>>;
    fn get_acl_token_self(
        &self,
        region: Option<&str>,
        namespace: Option<&str>,
        index: Option<i32>,
        wait: Option<&str>,
        stale: Option<&str>,
        prefix: Option<&str>,
        x_nomad_token: Option<&str>,
        per_page: Option<i32>,
        next_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<crate::models::AclToken, Error>>>>;
    fn get_acl_tokens(
        &self,
        region: Option<&str>,
        namespace: Option<&str>,
        index: Option<i32>,
        wait: Option<&str>,
        stale: Option<&str>,
        prefix: Option<&str>,
        x_nomad_token: Option<&str>,
        per_page: Option<i32>,
        next_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<crate::models::AclTokenListStub>, Error>>>>;
    fn post_acl_bootstrap(
        &self,
        region: Option<&str>,
        namespace: Option<&str>,
        x_nomad_token: Option<&str>,
        idempotency_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<crate::models::AclToken>, Error>>>>;
    fn post_acl_policy(
        &self,
        policy_name: &str,
        acl_policy: crate::models::AclPolicy,
        region: Option<&str>,
        namespace: Option<&str>,
        x_nomad_token: Option<&str>,
        idempotency_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>>>>;
    fn post_acl_token(
        &self,
        token_accessor: &str,
        acl_token: crate::models::AclToken,
        region: Option<&str>,
        namespace: Option<&str>,
        x_nomad_token: Option<&str>,
        idempotency_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<crate::models::AclToken, Error>>>>;
    fn post_acl_token_onetime(
        &self,
        region: Option<&str>,
        namespace: Option<&str>,
        x_nomad_token: Option<&str>,
        idempotency_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<crate::models::OneTimeToken, Error>>>>;
    fn post_acl_token_onetime_exchange(
        &self,
        one_time_token_exchange_request: crate::models::OneTimeTokenExchangeRequest,
        region: Option<&str>,
        namespace: Option<&str>,
        x_nomad_token: Option<&str>,
        idempotency_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<crate::models::AclToken, Error>>>>;
}

impl<C: hyper::client::connect::Connect> ACLApi for ACLApiClient<C>
where
    C: Clone + std::marker::Send + Sync,
{
    #[allow(unused_mut)]
    fn delete_acl_policy(
        &self,
        policy_name: &str,
        region: Option<&str>,
        namespace: Option<&str>,
        x_nomad_token: Option<&str>,
        idempotency_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>>>> {
        let mut req = __internal_request::Request::new(
            hyper::Method::DELETE,
            "/acl/policy/{policyName}".to_string(),
        )
        .with_auth(__internal_request::Auth::ApiKey(
            __internal_request::ApiKey {
                in_header: true,
                in_query: false,
                param_name: "X-Nomad-Token".to_owned(),
            },
        ));
        if let Some(ref s) = region {
            let query_value = s.to_string();
            req = req.with_query_param("region".to_string(), query_value);
        }
        if let Some(ref s) = namespace {
            let query_value = s.to_string();
            req = req.with_query_param("namespace".to_string(), query_value);
        }
        if let Some(ref s) = idempotency_token {
            let query_value = s.to_string();
            req = req.with_query_param("idempotency_token".to_string(), query_value);
        }
        req = req.with_path_param("policyName".to_string(), policy_name.to_string());
        if let Some(param_value) = x_nomad_token {
            req = req.with_header_param("X-Nomad-Token".to_string(), param_value.to_string());
        }
        req = req.returns_nothing();

        req.execute(self.configuration.borrow())
    }

    #[allow(unused_mut)]
    fn delete_acl_token(
        &self,
        token_accessor: &str,
        region: Option<&str>,
        namespace: Option<&str>,
        x_nomad_token: Option<&str>,
        idempotency_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>>>> {
        let mut req = __internal_request::Request::new(
            hyper::Method::DELETE,
            "/acl/token/{tokenAccessor}".to_string(),
        )
        .with_auth(__internal_request::Auth::ApiKey(
            __internal_request::ApiKey {
                in_header: true,
                in_query: false,
                param_name: "X-Nomad-Token".to_owned(),
            },
        ));
        if let Some(ref s) = region {
            let query_value = s.to_string();
            req = req.with_query_param("region".to_string(), query_value);
        }
        if let Some(ref s) = namespace {
            let query_value = s.to_string();
            req = req.with_query_param("namespace".to_string(), query_value);
        }
        if let Some(ref s) = idempotency_token {
            let query_value = s.to_string();
            req = req.with_query_param("idempotency_token".to_string(), query_value);
        }
        req = req.with_path_param("tokenAccessor".to_string(), token_accessor.to_string());
        if let Some(param_value) = x_nomad_token {
            req = req.with_header_param("X-Nomad-Token".to_string(), param_value.to_string());
        }
        req = req.returns_nothing();

        req.execute(self.configuration.borrow())
    }

    #[allow(unused_mut)]
    fn get_acl_policies(
        &self,
        region: Option<&str>,
        namespace: Option<&str>,
        index: Option<i32>,
        wait: Option<&str>,
        stale: Option<&str>,
        prefix: Option<&str>,
        x_nomad_token: Option<&str>,
        per_page: Option<i32>,
        next_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<crate::models::AclPolicyListStub>, Error>>>> {
        let mut req =
            __internal_request::Request::new(hyper::Method::GET, "/acl/policies".to_string())
                .with_auth(__internal_request::Auth::ApiKey(
                    __internal_request::ApiKey {
                        in_header: true,
                        in_query: false,
                        param_name: "X-Nomad-Token".to_owned(),
                    },
                ));
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
        if let Some(param_value) = index {
            req = req.with_header_param("index".to_string(), param_value.to_string());
        }
        if let Some(param_value) = x_nomad_token {
            req = req.with_header_param("X-Nomad-Token".to_string(), param_value.to_string());
        }

        req.execute(self.configuration.borrow())
    }

    #[allow(unused_mut)]
    fn get_acl_policy(
        &self,
        policy_name: &str,
        region: Option<&str>,
        namespace: Option<&str>,
        index: Option<i32>,
        wait: Option<&str>,
        stale: Option<&str>,
        prefix: Option<&str>,
        x_nomad_token: Option<&str>,
        per_page: Option<i32>,
        next_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<crate::models::AclPolicy, Error>>>> {
        let mut req = __internal_request::Request::new(
            hyper::Method::GET,
            "/acl/policy/{policyName}".to_string(),
        )
        .with_auth(__internal_request::Auth::ApiKey(
            __internal_request::ApiKey {
                in_header: true,
                in_query: false,
                param_name: "X-Nomad-Token".to_owned(),
            },
        ));
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
        req = req.with_path_param("policyName".to_string(), policy_name.to_string());
        if let Some(param_value) = index {
            req = req.with_header_param("index".to_string(), param_value.to_string());
        }
        if let Some(param_value) = x_nomad_token {
            req = req.with_header_param("X-Nomad-Token".to_string(), param_value.to_string());
        }

        req.execute(self.configuration.borrow())
    }

    #[allow(unused_mut)]
    fn get_acl_token(
        &self,
        token_accessor: &str,
        region: Option<&str>,
        namespace: Option<&str>,
        index: Option<i32>,
        wait: Option<&str>,
        stale: Option<&str>,
        prefix: Option<&str>,
        x_nomad_token: Option<&str>,
        per_page: Option<i32>,
        next_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<crate::models::AclToken, Error>>>> {
        let mut req = __internal_request::Request::new(
            hyper::Method::GET,
            "/acl/token/{tokenAccessor}".to_string(),
        )
        .with_auth(__internal_request::Auth::ApiKey(
            __internal_request::ApiKey {
                in_header: true,
                in_query: false,
                param_name: "X-Nomad-Token".to_owned(),
            },
        ));
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
        req = req.with_path_param("tokenAccessor".to_string(), token_accessor.to_string());
        if let Some(param_value) = index {
            req = req.with_header_param("index".to_string(), param_value.to_string());
        }
        if let Some(param_value) = x_nomad_token {
            req = req.with_header_param("X-Nomad-Token".to_string(), param_value.to_string());
        }

        req.execute(self.configuration.borrow())
    }

    #[allow(unused_mut)]
    fn get_acl_token_self(
        &self,
        region: Option<&str>,
        namespace: Option<&str>,
        index: Option<i32>,
        wait: Option<&str>,
        stale: Option<&str>,
        prefix: Option<&str>,
        x_nomad_token: Option<&str>,
        per_page: Option<i32>,
        next_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<crate::models::AclToken, Error>>>> {
        let mut req =
            __internal_request::Request::new(hyper::Method::GET, "/acl/token".to_string())
                .with_auth(__internal_request::Auth::ApiKey(
                    __internal_request::ApiKey {
                        in_header: true,
                        in_query: false,
                        param_name: "X-Nomad-Token".to_owned(),
                    },
                ));
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
        if let Some(param_value) = index {
            req = req.with_header_param("index".to_string(), param_value.to_string());
        }
        if let Some(param_value) = x_nomad_token {
            req = req.with_header_param("X-Nomad-Token".to_string(), param_value.to_string());
        }

        req.execute(self.configuration.borrow())
    }

    #[allow(unused_mut)]
    fn get_acl_tokens(
        &self,
        region: Option<&str>,
        namespace: Option<&str>,
        index: Option<i32>,
        wait: Option<&str>,
        stale: Option<&str>,
        prefix: Option<&str>,
        x_nomad_token: Option<&str>,
        per_page: Option<i32>,
        next_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<crate::models::AclTokenListStub>, Error>>>> {
        let mut req =
            __internal_request::Request::new(hyper::Method::GET, "/acl/tokens".to_string())
                .with_auth(__internal_request::Auth::ApiKey(
                    __internal_request::ApiKey {
                        in_header: true,
                        in_query: false,
                        param_name: "X-Nomad-Token".to_owned(),
                    },
                ));
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
        if let Some(param_value) = index {
            req = req.with_header_param("index".to_string(), param_value.to_string());
        }
        if let Some(param_value) = x_nomad_token {
            req = req.with_header_param("X-Nomad-Token".to_string(), param_value.to_string());
        }

        req.execute(self.configuration.borrow())
    }

    #[allow(unused_mut)]
    fn post_acl_bootstrap(
        &self,
        region: Option<&str>,
        namespace: Option<&str>,
        x_nomad_token: Option<&str>,
        idempotency_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<crate::models::AclToken>, Error>>>> {
        let mut req =
            __internal_request::Request::new(hyper::Method::POST, "/acl/bootstrap".to_string())
                .with_auth(__internal_request::Auth::ApiKey(
                    __internal_request::ApiKey {
                        in_header: true,
                        in_query: false,
                        param_name: "X-Nomad-Token".to_owned(),
                    },
                ));
        if let Some(ref s) = region {
            let query_value = s.to_string();
            req = req.with_query_param("region".to_string(), query_value);
        }
        if let Some(ref s) = namespace {
            let query_value = s.to_string();
            req = req.with_query_param("namespace".to_string(), query_value);
        }
        if let Some(ref s) = idempotency_token {
            let query_value = s.to_string();
            req = req.with_query_param("idempotency_token".to_string(), query_value);
        }
        if let Some(param_value) = x_nomad_token {
            req = req.with_header_param("X-Nomad-Token".to_string(), param_value.to_string());
        }

        req.execute(self.configuration.borrow())
    }

    #[allow(unused_mut)]
    fn post_acl_policy(
        &self,
        policy_name: &str,
        acl_policy: crate::models::AclPolicy,
        region: Option<&str>,
        namespace: Option<&str>,
        x_nomad_token: Option<&str>,
        idempotency_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>>>> {
        let mut req = __internal_request::Request::new(
            hyper::Method::POST,
            "/acl/policy/{policyName}".to_string(),
        )
        .with_auth(__internal_request::Auth::ApiKey(
            __internal_request::ApiKey {
                in_header: true,
                in_query: false,
                param_name: "X-Nomad-Token".to_owned(),
            },
        ));
        if let Some(ref s) = region {
            let query_value = s.to_string();
            req = req.with_query_param("region".to_string(), query_value);
        }
        if let Some(ref s) = namespace {
            let query_value = s.to_string();
            req = req.with_query_param("namespace".to_string(), query_value);
        }
        if let Some(ref s) = idempotency_token {
            let query_value = s.to_string();
            req = req.with_query_param("idempotency_token".to_string(), query_value);
        }
        req = req.with_path_param("policyName".to_string(), policy_name.to_string());
        if let Some(param_value) = x_nomad_token {
            req = req.with_header_param("X-Nomad-Token".to_string(), param_value.to_string());
        }
        req = req.with_body_param(acl_policy);
        req = req.returns_nothing();

        req.execute(self.configuration.borrow())
    }

    #[allow(unused_mut)]
    fn post_acl_token(
        &self,
        token_accessor: &str,
        acl_token: crate::models::AclToken,
        region: Option<&str>,
        namespace: Option<&str>,
        x_nomad_token: Option<&str>,
        idempotency_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<crate::models::AclToken, Error>>>> {
        let mut req = __internal_request::Request::new(
            hyper::Method::POST,
            "/acl/token/{tokenAccessor}".to_string(),
        )
        .with_auth(__internal_request::Auth::ApiKey(
            __internal_request::ApiKey {
                in_header: true,
                in_query: false,
                param_name: "X-Nomad-Token".to_owned(),
            },
        ));
        if let Some(ref s) = region {
            let query_value = s.to_string();
            req = req.with_query_param("region".to_string(), query_value);
        }
        if let Some(ref s) = namespace {
            let query_value = s.to_string();
            req = req.with_query_param("namespace".to_string(), query_value);
        }
        if let Some(ref s) = idempotency_token {
            let query_value = s.to_string();
            req = req.with_query_param("idempotency_token".to_string(), query_value);
        }
        req = req.with_path_param("tokenAccessor".to_string(), token_accessor.to_string());
        if let Some(param_value) = x_nomad_token {
            req = req.with_header_param("X-Nomad-Token".to_string(), param_value.to_string());
        }
        req = req.with_body_param(acl_token);

        req.execute(self.configuration.borrow())
    }

    #[allow(unused_mut)]
    fn post_acl_token_onetime(
        &self,
        region: Option<&str>,
        namespace: Option<&str>,
        x_nomad_token: Option<&str>,
        idempotency_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<crate::models::OneTimeToken, Error>>>> {
        let mut req =
            __internal_request::Request::new(hyper::Method::POST, "/acl/token/onetime".to_string())
                .with_auth(__internal_request::Auth::ApiKey(
                    __internal_request::ApiKey {
                        in_header: true,
                        in_query: false,
                        param_name: "X-Nomad-Token".to_owned(),
                    },
                ));
        if let Some(ref s) = region {
            let query_value = s.to_string();
            req = req.with_query_param("region".to_string(), query_value);
        }
        if let Some(ref s) = namespace {
            let query_value = s.to_string();
            req = req.with_query_param("namespace".to_string(), query_value);
        }
        if let Some(ref s) = idempotency_token {
            let query_value = s.to_string();
            req = req.with_query_param("idempotency_token".to_string(), query_value);
        }
        if let Some(param_value) = x_nomad_token {
            req = req.with_header_param("X-Nomad-Token".to_string(), param_value.to_string());
        }

        req.execute(self.configuration.borrow())
    }

    #[allow(unused_mut)]
    fn post_acl_token_onetime_exchange(
        &self,
        one_time_token_exchange_request: crate::models::OneTimeTokenExchangeRequest,
        region: Option<&str>,
        namespace: Option<&str>,
        x_nomad_token: Option<&str>,
        idempotency_token: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<crate::models::AclToken, Error>>>> {
        let mut req = __internal_request::Request::new(
            hyper::Method::POST,
            "/acl/token/onetime/exchange".to_string(),
        )
        .with_auth(__internal_request::Auth::ApiKey(
            __internal_request::ApiKey {
                in_header: true,
                in_query: false,
                param_name: "X-Nomad-Token".to_owned(),
            },
        ));
        if let Some(ref s) = region {
            let query_value = s.to_string();
            req = req.with_query_param("region".to_string(), query_value);
        }
        if let Some(ref s) = namespace {
            let query_value = s.to_string();
            req = req.with_query_param("namespace".to_string(), query_value);
        }
        if let Some(ref s) = idempotency_token {
            let query_value = s.to_string();
            req = req.with_query_param("idempotency_token".to_string(), query_value);
        }
        if let Some(param_value) = x_nomad_token {
            req = req.with_header_param("X-Nomad-Token".to_string(), param_value.to_string());
        }
        req = req.with_body_param(one_time_token_exchange_request);

        req.execute(self.configuration.borrow())
    }
}
