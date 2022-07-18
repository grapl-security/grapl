#![allow(warnings)]

use std::collections::HashSet;

use futures::future::try_join_all;
use rust_proto::graplinc::grapl::api::{
    graph_mutation::v1beta1::client::{
        GraphMutationClient,
        GraphMutationClientError,
    },
    lens_manager::v1beta1::{
        client::{
            LensManagerServiceClient,
            LensManagerServiceClientError,
        },
        messages::{
            AddNodeToScopeRequest,
            CreateLensRequest,
        },
    },
    plugin_sdk::analyzers::v1beta1::messages::{
        ExecutionHit,
        LensRef,
    },
};

#[derive(thiserror::Error, Debug)]
pub enum LensCreatorError {
    #[error("LensManagerServiceClientError {0}")]
    LensManagerServiceClientError(#[from] LensManagerServiceClientError),
}

#[derive(Clone)]
pub struct LensCreator {
    lens_manager_client: LensManagerServiceClient,
}

impl LensCreator {
    pub fn new(lens_manager_client: LensManagerServiceClient) -> Self {
        LensCreator {
            lens_manager_client,
        }
    }
}

impl LensCreator {
    pub async fn handle_event(
        &self,
        tenant_id: uuid::Uuid,
        execution_hit: ExecutionHit,
    ) -> Result<(), LensCreatorError> {
        // todo: We can cache th elens uids
        let mut lens_manager_client = self.lens_manager_client.clone();

        let ExecutionHit {
            graph_view,
            root_uid,
            lens_refs,
            analyzer_name,
            time_of_match,
            idempotency_key,
            score,
        } = execution_hit;
        let _root_uid = root_uid;
        let _analyzer_name = analyzer_name;
        let _time_of_match = time_of_match;
        let _idempotency_key = idempotency_key;
        let _score = score;

        let lens_uids = try_join_all(
            lens_refs
                .into_iter()
                .map(|lens_ref| (lens_ref, lens_manager_client.clone()))
                .map(|(lens_ref, mut lens_manager_client)| async move {
                    let LensRef {
                        lens_namespace,
                        lens_name,
                    } = lens_ref;

                    lens_manager_client
                        .create_lens(CreateLensRequest {
                            tenant_id,
                            lens_type: lens_namespace,
                            lens_name,
                            is_engagement: false,
                        })
                        .await
                }),
        )
        .await?;

        try_join_all(
            lens_uids
                .into_iter()
                .map(|lens_uid| (lens_uid, lens_manager_client.clone()))
                .flat_map(|(lens_uid, lens_manager_client)| {
                    let lens_uid = lens_uid.lens_uid;
                    graph_view
                        .nodes
                        .iter()
                        .map(move |(uid, node)| (uid, &node.node_type, lens_manager_client.clone()))
                        .map(
                            move |(uid, node_type, mut lens_manager_client)| async move {
                                // todo: We can cache that the node is already a part of the lens's scope
                                lens_manager_client
                                    .add_node_to_scope(AddNodeToScopeRequest {
                                        tenant_id,
                                        lens_uid,
                                        uid: uid.as_u64(),
                                        node_type: node_type.clone(),
                                    })
                                    .await
                            },
                        )
                }),
        )
        .await?;

        Ok(())
    }
}
