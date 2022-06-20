use std::sync::Arc;
use scylla::{CachingSession};
use scylla::cql_to_rust::FromRowError;
use scylla::transport::errors::QueryError;
use scylla::transport::query_result::MaybeFirstRowTypedError;
use rust_proto_new::graplinc::grapl::common::v1beta1::types::{EdgeName, NodeType, PropertyName, Uid};

#[derive(Debug, thiserror::Error)]
pub enum PropertyQueryError {
    #[error("QueryError: {0}")]
    DbError(#[from] QueryError),
    #[error("Too many rows: {0}")]
    MaybeFirstRowTypedError(#[from] MaybeFirstRowTypedError),
    #[error("Row was invalid {0}")]
    FromRowError(#[from] FromRowError),

}

#[derive(Debug, Clone)]
pub struct EdgeRow {
    pub source_uid: Uid,
    pub f_edge_name: EdgeName,
    pub r_edge_name: EdgeName,
    pub destination_uid: Uid,
    pub tenant_id: uuid::Uuid,
}


#[derive(Debug, Clone)]
pub struct StringField {
    pub uid: Uid,
    pub populated_field: PropertyName,
    pub value: String,
}


// We should push our filtering logic into here

#[derive(Clone)]
pub struct PropertyQueryExecutor {
    scylla_client: Arc<CachingSession>,
}

impl PropertyQueryExecutor {
    pub fn new(scylla_client: Arc<CachingSession>) -> Self {
        Self {
            scylla_client
        }
    }

    pub async fn get_immutable_string(
        &self,
        tenant_id: uuid::Uuid,
        uid: Uid,
        node_type: &NodeType,
        property_name: &PropertyName,
    ) -> Result<Option<StringField>, PropertyQueryError> {
        let tenant_urn = tenant_id.to_simple();

        let mut query = scylla::query::Query::from(
            format!(r"
            SELECT value
            FROM tenant_keyspace_{tenant_urn}.immutable_string_index
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

        Ok(Some(StringField {
            uid,
            populated_field: property_name.clone(),
            value: row,
        }))
    }


    pub async fn get_edges(
        &self,
        tenant_id: uuid::Uuid,
        uid: Uid,
        edge_name: &EdgeName,
    ) -> Result<Option<Vec<EdgeRow>>, PropertyQueryError> {
        let tenant_urn = tenant_id.to_simple();

        let mut query = scylla::query::Query::from(
            format!(r"
            SELECT r_edge_name, destination_uid
            FROM tenant_keyspace_{tenant_urn}.edge
            WHERE
                source_uid = ? AND
                f_edge_name = ?
            ALLOW FILTERING;
            ")
        );

        println!("query: \n{}\n", &query.contents);

        query.set_is_idempotent(true);

        let query_result = self.scylla_client.execute(
            query,
            &(uid.as_i64(), &edge_name.value)
        ).await?;


        let rows = query_result.rows_typed_or_empty::<(String, i64)>();

        let mut edge_rows = Vec::new();
        for row in rows {
            let (r_edge_name, destination_uid) = row?;
            edge_rows.push(
                    EdgeRow {
                        source_uid: uid,
                        f_edge_name: edge_name.clone(),
                        r_edge_name: EdgeName::try_from(r_edge_name).expect("todo"),
                        destination_uid: Uid::from_i64(destination_uid).expect("todo"),
                        tenant_id
                    }
            );
        }

        if edge_rows.is_empty() {
            Ok(None)
        } else {
            Ok(Some(edge_rows))
        }
    }
}