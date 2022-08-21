use rust_proto::graplinc::grapl::api::graph_schema_manager::v1beta1::messages::EdgeCardinality;

#[derive(sqlx::Type, Copy, Clone, Debug)]
#[sqlx(type_name = "edge_cardinality")]
pub enum StoredEdgeCardinality {
    ToOne,
    ToMany,
}

#[derive(sqlx::Type, Clone, Debug)]
pub struct GetEdgeSchemaRequestRow {
    pub reverse_edge_name: String,
    pub forward_edge_cardinality: StoredEdgeCardinality,
    pub reverse_edge_cardinality: StoredEdgeCardinality,
}

impl From<StoredEdgeCardinality> for EdgeCardinality {
    fn from(c: StoredEdgeCardinality) -> Self {
        match c {
            StoredEdgeCardinality::ToOne => EdgeCardinality::ToOne,
            StoredEdgeCardinality::ToMany => EdgeCardinality::ToMany,
        }
    }
}

#[derive(sqlx::Type, Copy, Clone, Debug)]
#[sqlx(type_name = "property_type")]
pub enum StoredPropertyType {
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
