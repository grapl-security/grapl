pub mod config;
pub mod db;
pub mod server;

use db::client::{
    SchemaDbClient,
    Txn,
};
use grapl_graphql_codegen::{
    conflict_resolution::ConflictResolution,
    edge::Edge,
    identification_algorithm::IdentificationAlgorithm,
    identity_predicate_type::IdentityPredicateType,
    node_predicate::NodePredicate,
    node_type,
    node_type::NodeType,
    parse_schema,
    predicate_type::PredicateType,
    Document,
    ParseError,
};
use sqlx::{
    Postgres,
    Transaction,
};

use crate::db::models::{
    StoredEdgeCardinality,
    StoredPropertyType,
};

const SCHEMA_TYPE: &str = "Graphql_V0";

#[derive(thiserror::Error, Debug)]
pub enum DeployGraphqlError {
    #[error("Failed to parse schema {0}")]
    ParseError(String),
    #[error("Failed to deploy schema due to sqlx error {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("Failed to deploy schema due to graphql error {0}")]
    GraphqlParseError(#[from] ParseError),
    #[error("Schema is invalid {0}")]
    InvalidSchema(&'static str),
}

pub async fn deploy_graphql_plugin(
    tenant_id: uuid::Uuid,
    raw_schema: &str,
    schema_version: u32,
    db_client: &SchemaDbClient,
) -> Result<(), DeployGraphqlError> {
    let document: Document<String> = parse_schema(raw_schema)?;
    let document = document.into_static();

    let node_types = node_type::parse_into_node_types(document)
        .map_err(|e| DeployGraphqlError::ParseError(e.to_string()))?;

    let mut txn = db_client.begin_txn().await?;

    for node_type in node_types.iter() {
        deploy_identity_algorithm(tenant_id, node_type, schema_version, &mut txn).await?;

        deploy_node_type(
            &mut txn,
            db_client,
            tenant_id,
            node_type,
            schema_version,
            raw_schema,
        )
        .await?;

        for property in node_type.predicates.iter() {
            deploy_node_property(
                &mut txn,
                db_client,
                tenant_id,
                node_type,
                property,
                schema_version,
            )
            .await?;
        }

        for edge in node_type.edges.iter() {
            deploy_edge(
                &mut txn,
                db_client,
                tenant_id,
                node_type,
                edge,
                schema_version,
            )
            .await?;
        }
    }

    txn.commit().await?;

    Ok(())
}

async fn deploy_node_type(
    txn: &mut Txn<'_>,
    db_client: &SchemaDbClient,
    tenant_id: uuid::Uuid,
    node_type: &NodeType,
    schema_version: u32,
    raw_schema: &str,
) -> Result<(), DeployGraphqlError> {
    let identity_algorithm = match node_type.identification_algorithm {
        IdentificationAlgorithm::Session => "session",
        IdentificationAlgorithm::Static => "static",
    };
    let node_type_name = &node_type.type_name;

    db_client
        .insert_node_schema(
            txn,
            tenant_id,
            identity_algorithm,
            node_type_name,
            schema_version,
            raw_schema,
            SCHEMA_TYPE,
        )
        .await?;

    Ok(())
}

async fn deploy_identity_algorithm(
    tenant_id: uuid::Uuid,
    node_type: &NodeType,
    schema_version: u32,
    txn: &mut Transaction<'_, Postgres>,
) -> Result<(), DeployGraphqlError> {
    let identity_algorithm = match node_type.identification_algorithm {
        IdentificationAlgorithm::Session => "session",
        IdentificationAlgorithm::Static => "static",
    };
    let node_type_name = &node_type.type_name;
    sqlx::query!(
        r#"
        INSERT INTO schema_manager.node_identity_algorithm (
            tenant_id,
            identity_algorithm,
            node_type,
            schema_version
        )
        VALUES ($1, $2, $3, $4)
        "#,
        tenant_id,
        identity_algorithm,
        node_type_name,
        schema_version as i16,
    )
    .execute(&mut *txn)
    .await?;

    match node_type.identification_algorithm {
        IdentificationAlgorithm::Session => {
            deploy_session_identity(tenant_id, node_type, schema_version, txn).await?;
        }
        IdentificationAlgorithm::Static => {
            deploy_static_identity(tenant_id, node_type, schema_version, txn).await?;
        }
    }

    Ok(())
}

async fn deploy_session_identity(
    tenant_id: uuid::Uuid,
    node_type: &NodeType,
    schema_version: u32,
    txn: &mut Transaction<'_, Postgres>,
) -> Result<(), DeployGraphqlError> {
    let node_type_name = &node_type.type_name;

    let mut pseudo_keys = Vec::with_capacity(1);

    let mut creation_timestamp_property: Option<String> = None;
    let mut last_seen_timestamp_property: Option<String> = None;
    let mut termination_timestamp_property: Option<String> = None;

    for field in node_type.predicates.iter() {
        match field.identity_predicate_type {
            Some(IdentityPredicateType::SessionPseudoKey) => {
                pseudo_keys.push(field.predicate_name.clone());
            }
            Some(IdentityPredicateType::SessionCreateTime) => {
                creation_timestamp_property = Some(field.predicate_name.to_string());
            }
            Some(IdentityPredicateType::SessionLastSeenTime) => {
                last_seen_timestamp_property = Some(field.predicate_name.to_string());
            }
            Some(IdentityPredicateType::SessionTerminateTime) => {
                termination_timestamp_property = Some(field.predicate_name.to_string());
            }
            Some(IdentityPredicateType::StaticId) => {
                return Err(DeployGraphqlError::InvalidSchema(
                    "StaticId is not allowed in session identity",
                ));
            }
            None => {}
        }
    }

    if pseudo_keys.is_empty() {
        return Err(DeployGraphqlError::InvalidSchema(
            "Session identity must have at least one pseudo key",
        ));
    }

    let creation_timestamp_property = creation_timestamp_property.ok_or_else(|| {
        DeployGraphqlError::InvalidSchema("creation_timestamp_property must be present")
    })?;
    let last_seen_timestamp_property = last_seen_timestamp_property.ok_or_else(|| {
        DeployGraphqlError::InvalidSchema("last_seen_timestamp_property must be present")
    })?;
    let termination_timestamp_property = termination_timestamp_property.ok_or_else(|| {
        DeployGraphqlError::InvalidSchema("termination_timestamp_property must be present")
    })?;

    sqlx::query!(
        r#"
        INSERT INTO schema_manager.session_identity_arguments (
            tenant_id,
            identity_algorithm,
            node_type,
            schema_version,
            pseudo_key_properties,
            negation_key_properties,
            creation_timestamp_property,
            last_seen_timestamp_property,
            termination_timestamp_property
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        tenant_id,
        "session",
        node_type_name,
        schema_version as i16,
        &pseudo_keys[..],
        &[][..], // todo: negation keys are not supported in the parser
        creation_timestamp_property,
        last_seen_timestamp_property,
        termination_timestamp_property
    )
    .execute(&mut *txn)
    .await?;

    Ok(())
}

async fn deploy_static_identity(
    tenant_id: uuid::Uuid,
    node_type: &NodeType,
    schema_version: u32,
    txn: &mut Transaction<'_, Postgres>,
) -> Result<(), DeployGraphqlError> {
    let node_type_name = &node_type.type_name;

    let mut static_keys = Vec::with_capacity(1);
    for field in node_type.predicates.iter() {
        match field.identity_predicate_type {
            Some(IdentityPredicateType::StaticId) => {
                static_keys.push(field.predicate_name.clone());
            }
            Some(IdentityPredicateType::SessionPseudoKey) => {
                return Err(DeployGraphqlError::InvalidSchema(
                    "SessionPseudoKey is not allowed in static identity",
                ));
            }
            Some(IdentityPredicateType::SessionCreateTime) => {
                return Err(DeployGraphqlError::InvalidSchema(
                    "SessionCreateTime is not allowed in static identity",
                ));
            }
            Some(IdentityPredicateType::SessionLastSeenTime) => {
                return Err(DeployGraphqlError::InvalidSchema(
                    "SessionLastSeenTime is not allowed in static identity",
                ));
            }
            Some(IdentityPredicateType::SessionTerminateTime) => {
                return Err(DeployGraphqlError::InvalidSchema(
                    "SessionTerminateTime is not allowed in static identity",
                ));
            }
            None => (),
        }
    }

    if static_keys.is_empty() {
        return Err(DeployGraphqlError::InvalidSchema(
            "At least one static_key must be present",
        ));
    }

    sqlx::query!(
        r#"
        INSERT INTO schema_manager.static_identity_arguments (
            tenant_id,
            identity_algorithm,
            node_type,
            schema_version,
            static_key_properties
        )
        VALUES ($1, $2, $3, $4, $5)
        "#,
        tenant_id,
        "static",
        node_type_name,
        schema_version as i16,
        &static_keys[..],
    )
    .execute(&mut *txn)
    .await?;

    Ok(())
}

async fn deploy_node_property(
    txn: &mut Transaction<'_, Postgres>,
    db_client: &SchemaDbClient,
    tenant_id: uuid::Uuid,
    node_type: &NodeType,
    property: &NodePredicate,
    schema_version: u32,
) -> Result<(), DeployGraphqlError> {
    let node_type_name = &node_type.type_name;
    let predicate_type_name =
        get_predicate_type_name(property.predicate_type, property.conflict_resolution)?;
    db_client
        .insert_node_property(
            txn,
            tenant_id,
            node_type_name,
            schema_version,
            &property.predicate_name,
            predicate_type_name,
        )
        .await?;

    Ok(())
}

async fn deploy_edge(
    txn: &mut Txn<'_>,
    db_client: &SchemaDbClient,
    tenant_id: uuid::Uuid,
    node_type: &NodeType,
    edge: &Edge,
    schema_version: u32,
) -> Result<(), DeployGraphqlError> {
    let forward_edge_cardinality = if edge.relationship.to_one() {
        StoredEdgeCardinality::ToOne
    } else {
        StoredEdgeCardinality::ToMany
    };

    let reverse_edge_cardinality = if edge.relationship.reverse().to_one() {
        StoredEdgeCardinality::ToOne
    } else {
        StoredEdgeCardinality::ToMany
    };

    db_client
        .insert_edge_schema(
            txn,
            tenant_id,
            &node_type.type_name,
            &edge.edge_name,
            forward_edge_cardinality,
            &edge.reverse_edge_name,
            reverse_edge_cardinality,
            schema_version,
        )
        .await?;
    Ok(())
}

fn get_predicate_type_name(
    predicate_type: PredicateType,
    conflict_resolution: ConflictResolution,
) -> Result<StoredPropertyType, DeployGraphqlError> {
    let type_name = match (predicate_type, conflict_resolution) {
        (PredicateType::String, ConflictResolution::Immutable) => {
            StoredPropertyType::ImmutableString
        }
        (PredicateType::I64, ConflictResolution::Immutable) => StoredPropertyType::ImmutableI64,
        (PredicateType::I64, ConflictResolution::IncrementOnly) => StoredPropertyType::MaxI64,
        (PredicateType::I64, ConflictResolution::DecrementOnly) => StoredPropertyType::MinI64,
        (PredicateType::U64, ConflictResolution::Immutable) => StoredPropertyType::ImmutableU64,
        (PredicateType::U64, ConflictResolution::IncrementOnly) => StoredPropertyType::MaxU64,
        (PredicateType::U64, ConflictResolution::DecrementOnly) => StoredPropertyType::MinU64,
        (PredicateType::String, ConflictResolution::IncrementOnly) => {
            return Err(DeployGraphqlError::InvalidSchema(
                "String can only be ImmutableString. Got IncrementOnly",
            ));
        }
        (PredicateType::String, ConflictResolution::DecrementOnly) => {
            return Err(DeployGraphqlError::InvalidSchema(
                "String can only be ImmutableString. Got DecrementOnly",
            ));
        }
    };

    Ok(type_name)
}
