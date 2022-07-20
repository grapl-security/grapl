use std::sync::Arc;

use blake2::{
    digest::consts::U8,
    Blake2b,
    Digest,
};
use futures::stream::StreamExt;
use rust_proto::{
    graplinc::grapl::api::lens_manager::v1beta1::{
        messages::{
            AddNodeToScopeRequest,
            AddNodeToScopeResponse,
            CloseLensRequest,
            CloseLensResponse,
            CreateLensRequest,
            CreateLensResponse,
            ListLensesRequest,
            ListLensesResponse,
            MergeBehavior,
            MergeLensRequest,
            MergeLensResponse,
            RemoveNodeFromAllScopesRequest,
            RemoveNodeFromAllScopesResponse,
            RemoveNodeFromScopeRequest,
            RemoveNodeFromScopeResponse,
        },
        server::LensManagerApi,
    },
    protocol::status::Status,
};
use scylla::{
    query::Query,
    transport::{
        errors::QueryError,
        iterator::NextRowError,
    },
    CachingSession,
};
use rust_proto::graplinc::grapl::api::lens_manager::v1beta1::messages::ListLensesEntry;
use rust_proto::graplinc::grapl::common::v1beta1::types::Uid;

type Blake2b8 = Blake2b<U8>;

#[derive(thiserror::Error, Debug)]
pub enum LensManagerServerError {
    #[error("QueryError {0}")]
    QueryError(#[from] QueryError),
    #[error("NextRowError {0}")]
    NextRowError(#[from] NextRowError),
    #[error("Invalid Uid")]
    InvalidUid,
}

impl From<LensManagerServerError> for Status {
    fn from(e: LensManagerServerError) -> Self {
        Status::internal(format!("LensManagerServerError: {:?}", e))
    }
}


#[derive(Clone)]
pub struct LensManager {
    scylla_client: Arc<CachingSession>,
}

impl LensManager {
    pub fn new(scylla_client: Arc<CachingSession>) -> Self {
        Self {
            scylla_client,
        }
    }
}

#[async_trait::async_trait]
impl LensManagerApi for LensManager {
    type Error = LensManagerServerError;

    async fn create_lens(
        &self,
        request: CreateLensRequest,
    ) -> Result<CreateLensResponse, Self::Error> {
        // todo: Cache/ short circuit this
        let lens_uid = make_lens_uid(&request.lens_type, &request.lens_name);

        let tenant_urn = request.tenant_id.simple();

        let mut insert = Query::from(format!(
            r#"
                INSERT INTO tenant_keyspace_{tenant_urn}.lenses (uid, namespace, name)
                VALUES (?, ?, ?)
            "#
        ));
        insert.set_is_idempotent(true);
        self.scylla_client
            .execute(
                insert,
                &(lens_uid as i64, request.lens_type, request.lens_name),
            )
            .await?;

        Ok(CreateLensResponse { lens_uid })
    }

    async fn merge_lens(
        &self,
        request: MergeLensRequest,
    ) -> Result<MergeLensResponse, Self::Error> {
        let MergeLensRequest {
            tenant_id,
            source_lens_uid,
            target_lens_uid,
            merge_behavior,
        } = request;
        let tenant_urn = tenant_id.simple();

        let mut select = Query::from(format!(
            r#"
                SELECT node_uid, node_type FROM tenant_keyspace_{tenant_urn}.scope
                WHERE lens_uid = ?
            "#
        ));
        select.set_is_idempotent(true);
        let mut rows = self
            .scylla_client
            .execute_iter(select, &(source_lens_uid as i64, ))
            .await?
            .into_typed::<(i64, String)>();
        while let Some(row) = rows.next().await {
            let (node_uid, node_type) = row?;
            let mut insert = Query::from(format!(
                r#"
                    INSERT INTO tenant_keyspace_{tenant_urn}.scope (lens_uid, node_uid, node_type)
                    VALUES (?, ?, ?)
                "#
            ));

            insert.set_is_idempotent(true);
            self.scylla_client
                .execute(insert, &(target_lens_uid as i64, node_uid, node_type))
                .await?;
        }

        if let MergeBehavior::Close = merge_behavior {
            self.close_lens(CloseLensRequest {
                tenant_id: request.tenant_id,
                lens_uid: source_lens_uid,
            })
                .await?;
        }

        Ok(MergeLensResponse {})
    }

    async fn close_lens(
        &self,
        request: CloseLensRequest,
    ) -> Result<CloseLensResponse, Self::Error> {
        let CloseLensRequest {
            tenant_id,
            lens_uid,
        } = request;
        let tenant_urn = tenant_id.simple();

        let mut delete_from_scope = Query::from(format!(
            r#"
                    DELETE FROM tenant_keyspace_{tenant_urn}.scope WHERE lens_uid = ?
                "#
        ));
        delete_from_scope.set_is_idempotent(true);
        self.scylla_client
            .execute(delete_from_scope, &(lens_uid as i64, ))
            .await?;

        let mut delete_lens = Query::from(format!(
            r#"
                    DELETE FROM tenant_keyspace_{tenant_urn}.lenses WHERE lens_uid = ?
                "#
        ));
        delete_lens.set_is_idempotent(true);
        self.scylla_client
            .execute(delete_lens, &(lens_uid as i64, ))
            .await?;

        Ok(CloseLensResponse {})
    }

    async fn add_node_to_scope(
        &self,
        request: AddNodeToScopeRequest,
    ) -> Result<AddNodeToScopeResponse, Self::Error> {
        let tenant_urn = request.tenant_id.simple();
        let mut insert_scope = Query::from(format!(
            r#"
                INSERT INTO tenant_keyspace_{tenant_urn}.scope (lens_uid, node_uid, node_type)
                VALUES (?, ?, ?)
            "#
        ));

        insert_scope.set_is_idempotent(true);

        self.scylla_client
            .execute(
                insert_scope,
                &(
                    request.lens_uid as i64,
                    request.uid as i64,
                    request.node_type.value,
                ),
            )
            .await?;

        Ok(AddNodeToScopeResponse {})
    }

    async fn remove_node_from_scope(
        &self,
        request: RemoveNodeFromScopeRequest,
    ) -> Result<RemoveNodeFromScopeResponse, Self::Error> {
        let RemoveNodeFromScopeRequest {
            tenant_id,
            lens_uid,
            uid,
        } = request;
        let tenant_urn = tenant_id.simple();

        let mut delete_from_scope = Query::from(format!(
            r#"
                DELETE FROM tenant_keyspace_{tenant_urn}.scope
                WHERE lens_uid = ? AND
                      node_uid = ?
            "#
        ));
        delete_from_scope.set_is_idempotent(true);
        self.scylla_client
            .execute(delete_from_scope, &(lens_uid as i64, uid as i64))
            .await?;

        Ok(RemoveNodeFromScopeResponse {})
    }

    async fn remove_node_from_all_scopes(
        &self,
        request: RemoveNodeFromAllScopesRequest,
    ) -> Result<RemoveNodeFromAllScopesResponse, Self::Error> {
        let tenant_urn = request.tenant_id.simple();
        let mut scopes = self
            .scylla_client
            .execute_iter(
                Query::from(format!(
                    r#"
                    SELECT lens_uid FROM tenant_keyspace_{tenant_urn}.scope
                    WHERE node_uid = ?
                "#
                )),
                &(request.uid as i64, ),
            )
            .await?
            .into_typed::<(i64, )>();

        while let Some(scope) = scopes.next().await {
            let (lens_uid, ) = scope?;
            self.remove_node_from_scope(RemoveNodeFromScopeRequest {
                tenant_id: request.tenant_id,
                lens_uid: lens_uid as u64,
                uid: request.uid,
            })
                .await?;
        }

        Ok(RemoveNodeFromAllScopesResponse {})
    }

    /// List all lenses for a tenant
    async fn list_lenses(
        &self,
        request: ListLensesRequest,
    ) -> Result<ListLensesResponse, Self::Error> {
        let
            ListLensesRequest { tenant_id, offset, limit } = request;
        let tenant_urn = tenant_id.simple();

        let mut select = Query::from(format!(
            r#"
                SELECT lens_uid FROM tenant_keyspace_{tenant_urn}.scope
                WHERE lens_uid > ?
                LIMIT ?
            "#
        ));

        let mut rows = self
            .scylla_client
            .execute_iter(select, &(offset, limit))
            .await?
            .into_typed::<(i64, )>();

        let mut lenses = Vec::new();
        let mut last_uid = 0;
        while let Some(row) = rows.next().await {
            let lens_uid = Uid::from_i64(row?.0);
            match lens_uid {
                None => { return Err(Self::Error::InvalidUid); }
                Some(lens_uid) => {
                    lenses.push(ListLensesEntry { lens_uid });
                    last_uid = std::cmp::max(last_uid, lens_uid);
                }
            }
        }

        Ok(ListLensesResponse {
            lenses,
            offset: last_uid as u64,
        })
    }
}

fn make_lens_uid(namespace: &str, name: &str) -> u64 {
    let mut hasher = Blake2b8::new();
    hasher.update(namespace);
    hasher.update(name);
    let hash = hasher.finalize();
    u64::from_le_bytes(hash.into()) | 1
}

// async fn provision_lens_table(
//     session: &Session,
//     tenant_id: uuid::Uuid,
// ) -> Result<(), LensManagerServerError> {
//     session.query(
//         format!(
//             "
//                 CREATE TABLE IF NOT EXISTS tenant_keyspace_{}.lenses (
//                        uid bigint,
//                        namespace text,
//                        name text,
//                        score bigint,
//                        PRIMARY KEY (uid, score)
//                        CLUSTERING ORDER (uid ASC, score DESC)
//                 )
//                 WITH compression = {{
//                     'sstable_compression': 'LZ4Compressor',
//                     'chunk_length_in_kb': 64
//                 }};
//
//                 CREATE TABLE IF NOT EXISTS tenant_keyspace_{}.scope (
//                        lens_uid bigint,
//                        node_uid bigint,
//                        node_type text,
//                        PRIMARY KEY (lens_uid, node_uid)
//                 )
//                 WITH compression = {{
//                     'sstable_compression': 'LZ4Compressor',
//                     'chunk_length_in_kb': 64
//                 }};
//
//                 CREATE INDEX ON tenant_keyspace_{}.scope (node_uid);
//
//                 CREATE TABLE IF NOT EXISTS tenant_keyspace_{}.analyzer_matches (
//                        analyzer_name text,
//                        idempotency_key bigint,
//                        score int,
//                        PRIMARY KEY (analyzer_name, idempotency_key)
//                 )
//                 WITH compression = {{
//                     'sstable_compression': 'LZ4Compressor'
//                 }};
//                 ",
//             tenant_id.simple(),
//             tenant_id.simple(),
//             tenant_id.simple(),
//             tenant_id.simple(),
//         ),
//         &[],
//     );
//
//     Ok(())
// }
