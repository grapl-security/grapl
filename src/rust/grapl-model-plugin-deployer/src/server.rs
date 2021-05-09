use tonic::{transport::Server, Request, Response, Status};

use crate::grapl_model_plugin_deployer::GraplModelPluginDeployerRequest;
pub use crate::grapl_model_plugin_deployer::GraplModelPluginDeployerResponse;
pub use crate::grapl_model_plugin_deployer::grapl_model_plugin_deployer_rpc_server::GraplModelPluginDeployerRpc;
pub use crate::grapl_model_plugin_deployer::grapl_model_plugin_deployer_rpc_server::GraplModelPluginDeployerRpcServer;
use tracing::Span;
use rusoto_s3::{S3Client, S3, PutObjectRequest};
use grapl_config::env_helpers::FromEnv;
use rusoto_dynamodb::{DynamoDbClient, PutItemInput, AttributeValue};
use std::collections::HashMap;
use grapl_graphql_codegen::predicate_type::PredicateType;

fn standin_imports() -> String {
    let mut code = String::new();
    code.push_str("from __future__ import annotations\n");
    code.push_str("from typing import *\n");
    code.push_str("import grapl_analyzerlib\n");
    code.push_str("import grapl_analyzerlib.node_types\n");
    code.push_str("import grapl_analyzerlib.nodes.entity\n");
    code.push_str("import grapl_analyzerlib.queryable\n");
    code
}

pub struct GraplModelPluginDeployer {
    s3_client: S3Client,
    dynamodb_client: DynamoDbClient,
    model_plugins_bucket: String,
}

impl GraplModelPluginDeployer {
    fn new() -> Self {
        Self {
            s3_client: S3Client::from_env(),
            dynamodb_client: DynamoDbClient::from_env(),
            model_plugins_bucket: grapl_config::grapl_model_plugin_bucket()
        }
    }

    fn generate_code<'a>(&self, document: &grapl_graphql_codegen::node_type::Document<'a, &'a str>) -> String {
        let node_types = grapl_graphql_codegen::node_type::parse_into_node_types(&document).expect("Failed");

        let mut all_code = String::with_capacity(1024 * node_types.len());
        all_code.push_str(&standin_imports());
        for node_type in node_types {
            let pycode = node_type.generate_python_code();
            all_code.push_str(&pycode);
        }

        all_code
    }

    async fn deploy_plugin(
        &self,
        model_plugin_schema: &str,
        schema_version: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let document = grapl_graphql_codegen::parse_schema(model_plugin_schema)?;
        let plugin_file = self.generate_code(&document);
        self.upload_plugin_file(plugin_file, schema_version).await?;
        Ok(())
    }

    async fn push_schemas<'a>(&self, document: &grapl_graphql_codegen::node_type::Document<'a, &'a str>) -> Result<(), Box<dyn std::error::Error>> {
        let node_types = grapl_graphql_codegen::node_type::parse_into_node_types(&document).expect("Failed");
        let mut predicate_schemas = Vec::with_capacity(node_types.len() * 2);
        for node_type in node_types {
            let mut item: HashMap<String, AttributeValue> = HashMap::new();
            for predicate in node_type.predicates {
                let type_name = match predicate.predicate_type {
                    PredicateType::String => "String",
                    PredicateType::I64 => "Int",
                    PredicateType::U64 => "Int",
                };
                item.insert("name".to_string(), AttributeValue {
                    s: Some(predicate.predicate_name),
                    ..Default::default()
                });
                item.insert("primitive".to_string(), AttributeValue {
                    s: Some(type_name.to_string()),
                    ..Default::default()
                });
                item.insert("is_set".to_string(), AttributeValue {
                    s: Some(todo!("We don't currently track if a predicate is a set")),
                    ..Default::default()
                });
            }
            let input = PutItemInput {
                table_name: "node_schemas".to_string(),
                item,
                ..PutItemInput::default()
            };
            predicate_schemas.push(input);
        }
        Ok(())
    }

    async fn upload_plugin_file(&self, plugin_file: String, schema_version: i32) -> Result<(), Box<dyn std::error::Error>> {
        let key = format!("model_plugins/python/v{}/plugins.py", schema_version);

        self.s3_client.put_object(
            PutObjectRequest {
                body: Some(plugin_file.into_bytes().into()),
                bucket: self.model_plugins_bucket.clone(),
                key,
                ..Default::default()
            }
        ).await?;
        todo!("Write the plugin file out to s3");
        // Ok(())
    }
}

#[tonic::async_trait]
impl GraplModelPluginDeployerRpc for GraplModelPluginDeployer {
    #[tracing::instrument(
    remote_address = ? request.remote_addr(),
    trace_id = ? uuidv4(),
    record = field::Empty,
    request_id = field::Empty,
    skip(self, request)
    )]
    async fn deploy_plugin(
        &self,
        request: Request<GraplModelPluginDeployerRequest>,
    ) -> Result<Response<GraplModelPluginDeployerResponse>, Status> {
        let request = request.into_inner();
        Span::current()
            .record("client_name", &request.client_name.as_str())
            .record("request_id", &request.request_id.as_str());
        let _ = self.deploy_plugin(&request.model_plugin_schema, request.schema_version);
        let reply = GraplModelPluginDeployerResponse {};
        Ok(Response::new(reply))
    }
}

pub async fn exec_service() -> Result<(), Box<dyn std::error::Error>> {
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<GraplModelPluginDeployerRpcServer<GraplModelPluginDeployer>>()
        .await;

    let addr = "[::1]:50051".parse().unwrap();
    let _span = tracing::info_span!("service_exec", addr=&format!("{:?}", addr).as_str()).entered();

    let grapl_model_plugin_deployer_instance = GraplModelPluginDeployer::new();

    tracing::info!(
        message="HealthServer + GraplModelPluginDeployer listening",
    );

    Server::builder()
        .add_service(health_service)
        .add_service(GraplModelPluginDeployerRpcServer::new(grapl_model_plugin_deployer_instance))
        .serve(addr)
        .await?;

    Ok(())
}
