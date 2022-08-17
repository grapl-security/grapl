pub mod config;
pub mod db;
pub mod server;

use grapl_graphql_codegen::{
    conflict_resolution::ConflictResolution,
    edge::Edge as EdgeSchema,
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
    PgPool,
    Postgres,
    Transaction,
};

use crate::db::models::StoredEdgeCardinality;

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
    pool: &PgPool,
) -> Result<(), DeployGraphqlError> {
    let document: Document<String> = parse_schema(raw_schema)?;
    let document = document.into_static();

    let node_types = node_type::parse_into_node_types(document)
        .map_err(|e| DeployGraphqlError::ParseError(e.to_string()))?;

    let mut txn = pool.begin().await?;

    for node_type in node_types.iter() {
        deploy_identity_algorithm(tenant_id, node_type, schema_version, &mut txn).await?;

        deploy_node_type(tenant_id, node_type, schema_version, raw_schema, &mut txn).await?;

        for property in node_type.predicates.iter() {
            deploy_node_property(tenant_id, node_type, property, schema_version, &mut txn).await?;
        }

        for edge in node_type.edges.iter() {
            deploy_edge(tenant_id, node_type, edge, schema_version, &mut txn).await?;
        }
    }

    txn.commit().await?;

    Ok(())
}

async fn deploy_node_type(
    tenant_id: uuid::Uuid,
    node_type: &NodeType,
    schema_version: u32,
    raw_schema: &str,
    txn: &mut Transaction<'_, Postgres>,
) -> Result<(), DeployGraphqlError> {
    let identity_algorithm = match node_type.identification_algorithm {
        IdentificationAlgorithm::Session => "session",
        IdentificationAlgorithm::Static => "static",
    };
    let node_type_name = &node_type.type_name;

    sqlx::query!(
        r#"
        INSERT INTO schema_manager.node_schemas (
            tenant_id,
            identity_algorithm,
            node_type,
            schema_version,
            raw_schema,
            schema_type
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        tenant_id,
        identity_algorithm,
        node_type_name,
        schema_version as i16,
        raw_schema.as_bytes(),
        SCHEMA_TYPE,
    )
    .execute(&mut *txn)
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
    tenant_id: uuid::Uuid,
    node_type: &NodeType,
    property: &NodePredicate,
    schema_version: u32,
    txn: &mut Transaction<'_, Postgres>,
) -> Result<(), DeployGraphqlError> {
    let node_type_name = &node_type.type_name;
    let predicate_type_name =
        get_predicate_type_name(property.predicate_type, property.conflict_resolution)?;
    sqlx::query!(
        r#"
        INSERT INTO schema_manager.property_schemas (
            tenant_id,
            node_type,
            schema_version,
            property_name,
            property_type,
            identity_only
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        tenant_id,
        node_type_name,
        schema_version as i16,
        property.predicate_name,
        predicate_type_name as StoredPropertyType,
        false, // todo: implement identification only properties
    )
    .execute(&mut *txn)
    .await?;

    Ok(())
}

async fn deploy_edge(
    tenant_id: uuid::Uuid,
    node_type: &NodeType,
    edge: &EdgeSchema,
    schema_version: u32,
    txn: &mut Transaction<'_, Postgres>,
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

    sqlx::query!(
        r#"
        INSERT INTO schema_manager.edge_schemas (
            tenant_id,
            node_type,
            schema_version,
            forward_edge_name,
            reverse_edge_name,
            forward_edge_cardinality,
            reverse_edge_cardinality
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        tenant_id,
        &node_type.type_name,
        schema_version as i16,
        edge.edge_name,
        edge.reverse_edge_name,
        forward_edge_cardinality as StoredEdgeCardinality,
        reverse_edge_cardinality as StoredEdgeCardinality,
    )
    .execute(&mut *txn)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO schema_manager.edge_schemas (
            tenant_id,
            node_type,
            schema_version,
            forward_edge_name,
            reverse_edge_name,
            forward_edge_cardinality,
            reverse_edge_cardinality
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        tenant_id,
        &node_type.type_name,
        schema_version as i16,
        edge.reverse_edge_name,
        edge.edge_name,
        reverse_edge_cardinality as StoredEdgeCardinality,
        forward_edge_cardinality as StoredEdgeCardinality,
    )
    .execute(&mut *txn)
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

#[derive(sqlx::Type, Copy, Clone, Debug)]
#[sqlx(type_name = "property_type")]
enum StoredPropertyType {
    ImmutableString,
    ImmutableI64,
    MaxI64,
    MinI64,
    ImmutableU64,
    MaxU64,
    MinU64,
}

#[derive(sqlx::Type, Clone, Debug)]
struct NodeIdentityRow {
    identity_algorithm: String,
    tenant_id: uuid::Uuid,
    node_type: String,
    schema_version: i16,
}

#[derive(sqlx::Type, Clone, Debug)]
struct SessionIdentityRow {
    tenant_id: uuid::Uuid,
    identity_algorithm: String,
    node_type: String,
    schema_version: i16,
    pseudo_key_properties: Vec<String>,
    negation_key_properties: Vec<String>,
    creation_timestamp_property: String,
    last_seen_timestamp_property: String,
    termination_timestamp_property: String,
}

#[derive(sqlx::Type, Clone, Debug)]
struct NodeSchemaRow {
    tenant_id: sqlx::types::uuid::Uuid,
    identity_algorithm: String,
    node_type: String,
    schema_version: i16,
    deployment_timestamp: sqlx::types::time::PrimitiveDateTime,
    schema_type: String,
}

#[derive(sqlx::Type, Clone, Debug)]
struct PropertySchemaRow {
    tenant_id: sqlx::types::uuid::Uuid,
    node_type: String,
    schema_version: i16,
    property_name: String,
    property_type: StoredPropertyType,
    identity_only: bool,
}

#[derive(sqlx::Type, Clone, Debug)]
struct EdgeSchemaRow {
    tenant_id: sqlx::types::uuid::Uuid,
    node_type: String,
    schema_version: i16,
    forward_edge_name: String,
    reverse_edge_name: String,
    forward_edge_cardinality: StoredEdgeCardinality,
    reverse_edge_cardinality: StoredEdgeCardinality,
}
