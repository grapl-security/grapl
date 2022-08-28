use std::sync::Arc;

use rust_proto::{
    graplinc::grapl::common::v1beta1::types::{
        EdgeName,
        PropertyName,
        Uid,
    },
    SerDeError,
};
use scylla::{
    cql_to_rust::FromRowError,
    transport::{
        errors::QueryError,
        query_result::MaybeFirstRowTypedError,
    },
    CachingSession,
};

use crate::table_names::{
    tenant_keyspace_name,
    IMM_STRING_TABLE_NAME,
};

#[derive(Debug, thiserror::Error)]
pub enum PropertyQueryError {
    #[error("QueryError: {0}")]
    DbError(#[from] QueryError),
    #[error("Too many rows: {0}")]
    MaybeFirstRowTypedError(#[from] MaybeFirstRowTypedError),
    #[error("Row was invalid {0}")]
    FromRowError(#[from] FromRowError),
    #[error("Invalid destination_uid '{destination_uid}' (source_uid: '{source_uid:?}', f_edge_name: '{f_edge_name}')")]
    InvalidUidInDb {
        destination_uid: i64,
        source_uid: Uid,
        f_edge_name: String,
    },
    #[error("Invalid stored edge name {0}")]
    InvalidStoredEdgeName(#[from] SerDeError),
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
        Self { scylla_client }
    }

    pub async fn get_immutable_string(
        &self,
        tenant_id: uuid::Uuid,
        uid: Uid,
        property_name: &PropertyName,
    ) -> Result<Option<StringField>, PropertyQueryError> {
        let tenant_ks = tenant_keyspace_name(tenant_id);

        let mut query = scylla::query::Query::from(format!(
            r"
            SELECT value
            FROM {tenant_ks}.{IMM_STRING_TABLE_NAME}
            WHERE
                uid = ? AND
                populated_field = ?
            LIMIT 1
            ALLOW FILTERING;
            "
        ));

        query.set_is_idempotent(true);

        let query_result = self
            .scylla_client
            .execute(query, &(uid.as_i64(), &property_name.value))
            .await?;

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
        let tenant_ks = tenant_keyspace_name(tenant_id);

        let mut query = scylla::query::Query::from(format!(
            r"
            SELECT r_edge_name, destination_uid
            FROM {tenant_ks}.edges
            WHERE
                source_uid = ? AND
                f_edge_name = ?
            ALLOW FILTERING;
            "
        ));

        println!("query: \n{}\n", &query.contents);

        query.set_is_idempotent(true);

        let query_result = self
            .scylla_client
            .execute(query, &(uid.as_i64(), &edge_name.value))
            .await?;

        let rows = query_result.rows_typed_or_empty::<(String, i64)>();

        let mut edge_rows = Vec::new();
        for row in rows {
            let (r_edge_name, destination_uid) = row?;
            edge_rows.push(EdgeRow {
                source_uid: uid,
                f_edge_name: edge_name.clone(),
                r_edge_name: EdgeName::try_from(r_edge_name)
                    .map_err(PropertyQueryError::InvalidStoredEdgeName)?,
                destination_uid: Uid::from_i64(destination_uid).ok_or_else(|| {
                    PropertyQueryError::InvalidUidInDb {
                        destination_uid,
                        source_uid: uid,
                        f_edge_name: edge_name.to_string(),
                    }
                })?,
                tenant_id,
            });
        }

        if edge_rows.is_empty() {
            Ok(None)
        } else {
            Ok(Some(edge_rows))
        }
    }
}
