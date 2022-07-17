use std::pin::Pin;


use futures::Stream;

use sqlx::PgPool;

use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream};
use tracing::{info_span, Instrument};
use uuid::Uuid;

use rust_proto::graplinc::grapl::api::lens_subscription_service::v1beta1::messages::{SubscribeToLensRequest, SubscribeToLensResponse};
use rust_proto::graplinc::grapl::api::lens_subscription_service::v1beta1::server::LensSubscriptionApi;
use rust_proto::protocol::status::Status;
use rust_proto::graplinc::grapl::api::lens_subscription_service::v1beta1::messages::LensUpdate;
use rust_proto::graplinc::grapl::common::v1beta1::types::Uid;
use rust_proto::{SerDe, SerDeError};
use crate::notify_broadcast::{LensUpdateRow, NotifyBroadcaster, NotifyBroadcasterInitError};

pub type ResponseStream = Pin<Box<dyn Stream<Item=Result<SubscribeToLensResponse, LensSubscriptionServiceError>> + Send>>;
pub type SubscribeToLensResult<T> = Result<T, LensSubscriptionServiceError>;

const CHANNEL_NAME: &str = "lens_cdc";

#[derive(thiserror::Error, Debug)]
pub enum LensSubscriptionServiceError {
    #[error("Initial table scan failed: {0}")]
    InitialTableScan(#[source] sqlx::Error),
    #[error("Proto deserialize error: {0}")]
    LensUpdateDeserialize(#[source] SerDeError),
}

impl From<LensSubscriptionServiceError> for Status {
    fn from(_: LensSubscriptionServiceError) -> Self {
        todo!()
    }
}


#[derive(Clone)]
pub struct LensSubscriptionService {
    pool: PgPool,
    notify_broadcaster: NotifyBroadcaster,
}

impl LensSubscriptionService {
    pub async fn new(pool: PgPool) -> Result<Self, NotifyBroadcasterInitError> {
        let notify_broadcaster = NotifyBroadcaster::listen(&pool, CHANNEL_NAME).await?;
        Ok(Self {
            pool,
            notify_broadcaster,
        })
    }
}


#[async_trait::async_trait]
impl LensSubscriptionApi for LensSubscriptionService {
    type Error = LensSubscriptionServiceError;
    type SubscribeToLensStream = ResponseStream;

    #[tracing::instrument(skip(self), err)]
    async fn subscribe_to_lens(&self, request: SubscribeToLensRequest) -> SubscribeToLensResult<Self::SubscribeToLensStream> {
        // Important to start the subscription *before* we fetch our initial rows, otherwise
        // we can miss updates
        let mut brx = self.notify_broadcaster.subscribe(request.tenant_id, request.lens_uid);
        let (tx, rx) = mpsc::channel(5000);

        let initial_rows = initial_query(&self.pool, request.tenant_id, request.lens_uid).await?;

        tokio::task::spawn(async move {
            let mut latest_offset = 0i64;
            for row in initial_rows {
                latest_offset = std::cmp::max(latest_offset, row.update_offset);
                let lens_update = LensUpdate::deserialize(&row.lens_update[..])
                    .map(|lens_update| {
                        SubscribeToLensResponse {
                            lens_update,
                            update_offset: row.update_offset as u64,
                        }
                    })
                    .map_err(LensSubscriptionServiceError::LensUpdateDeserialize);

                // This will only ever be an error if the client has closed the connection
                if let Err(e) = tx.send(lens_update).await {
                    tracing::debug!(
                        message="Failed to send update",
                        error=?e,
                    );
                }
            }

            while let Ok((lens_update, update_offset)) = brx.recv().await {
                let _ = tx.send(Ok(SubscribeToLensResponse {
                    lens_update,
                    update_offset: update_offset as u64,
                }));
            }
        }.instrument(info_span!("subscribe_to_lens_loop")));

        let output_stream = ReceiverStream::new(rx);
        let s = Box::pin(output_stream) as Self::SubscribeToLensStream;
        Ok(s)
    }
}

struct MinLensUpdateRow {
    lens_update: Vec<u8>,
    update_offset: i64,
}

async fn initial_query(pool: &PgPool, tenant_id: Uuid, lens_uid: Uid) -> Result<Vec<LensUpdateRow>, LensSubscriptionServiceError> {
    let lens_uid = lens_uid.as_i64();

    let updates = sqlx::query_as!(
        MinLensUpdateRow,
        r"SELECT lens_update, update_offset
         FROM lens_subscriptions.lens_cdc WHERE
         tenant_id = $1 AND
         lens_uid = $2
         ORDER BY update_offset ASC",
        tenant_id,
        lens_uid
    )
        .fetch_all(pool)
        .await
        .map_err(LensSubscriptionServiceError::InitialTableScan)?
        .into_iter()
        .map(|update| LensUpdateRow {
            tenant_id,
            lens_update: update.lens_update,
            update_offset: update.update_offset,
            lens_uid,
        }).collect();

    Ok(updates)
}