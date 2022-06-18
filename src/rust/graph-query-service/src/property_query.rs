use std::sync::Arc;
use scylla::{CachingSession};
use scylla::transport::errors::QueryError;
use scylla::transport::query_result::MaybeFirstRowTypedError;
use rust_proto_new::graplinc::grapl::common::v1beta1::types::{NodeType, PropertyName, Uid};

#[derive(Debug, thiserror::Error)]
pub enum PropertyQueryError {
    #[error("QueryError: {0}")]
    DbError(#[from] QueryError),
    #[error("Too many rows: {0}")]
    MaybeFirstRowTypedError(#[from] MaybeFirstRowTypedError),
}

// We should push our filtering logic into here

#[derive(Clone)]
pub struct PropertyQuery {
    scylla_client: Arc<CachingSession>,
}

impl PropertyQuery {
    pub async fn get_immutable_string(
        &self,
        tenant_id: uuid::Uuid,
        uid: Uid,
        node_type: &NodeType,
        property_name: &PropertyName,
    ) -> Result<Option<String>, PropertyQueryError> {
        let tenant_urn = tenant_id.to_urn();

        let mut query = scylla::query::Query::from(
            format!(r"
            SELECT (value)
            FROM tenant_keyspace_{tenant_urn}
            WHERE
                uid = ? AND
                node_type = ? AND
                populated_field = ?
            LIMIT 1
            ALLOW FILTERING;
            ")
        );

        query.set_is_idempotent(true);

        let query_result = self.scylla_client.execute(
            query,
            &(uid.as_i64(), &node_type.value, &property_name.value)
        ).await?;

        let row = match query_result.maybe_first_row_typed::<(String,)>()? {
            Some((row,)) => row,
            None => return Ok(None),
        };


        Ok(Some(row))
    }
}