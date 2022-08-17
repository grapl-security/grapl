use rust_proto::graplinc::grapl::api::schema_manager::v1beta1::messages::EdgeCardinality;

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
