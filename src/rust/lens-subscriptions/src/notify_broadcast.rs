use std::sync::Arc;

use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::postgres::PgListener;
use uuid::Uuid;
use rust_proto::graplinc::grapl::common::v1beta1::types::Uid;
use tokio::sync::broadcast::{Sender as BroadcastSender, Receiver as BroadcastReceiver, channel as broadcast_channel};
use tokio::task::JoinHandle;
use tracing::Instrument;
use rust_proto::graplinc::grapl::api::lens_subscription_service::v1beta1::messages::LensUpdate;

use rust_proto::SerDe;

type TenantId = Uuid;
pub type LensUpdateReceiver = BroadcastReceiver<(LensUpdate, i64)>;
type Subscriptions = DashMap<(TenantId, i64), BroadcastSender<(LensUpdate, i64)>>;

#[derive(thiserror::Error, Debug)]
pub enum NotifyBroadcasterInitError {
    #[error("Failed to acquire connection to Postgres: {source}")]
    AcquireConnection {
        #[source]
        source: sqlx::Error,
    },
    #[error("Failed to initialize Postgres listener: {source}")]
    FailedListener {
        #[source]
        source: sqlx::Error,
    },
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct LensUpdateRow {
    pub tenant_id: TenantId,
    pub lens_uid: i64,
    pub lens_update: Vec<u8>,
    pub update_offset: i64,
}

#[derive(Clone)]
pub struct NotifyBroadcaster {
    subscriptions: Subscriptions,
    listener_task: Arc<JoinHandle<()>>,
}

impl NotifyBroadcaster {
    /// Ensure your listener is already listening
    pub async fn listen(pool: &PgPool, channel: &str) -> Result<Self, NotifyBroadcasterInitError> {
        let mut listener = PgListener::connect_with(&pool).await
            .map_err(|e| NotifyBroadcasterInitError::AcquireConnection { source: e })?;

        listener.listen(channel).await
            .map_err(|e| NotifyBroadcasterInitError::FailedListener { source: e })?;

        let subscriptions: Subscriptions = DashMap::new();
        let listener_task = Arc::new(listener_loop(listener, subscriptions.clone()));
        Ok(Self {
            subscriptions,
            listener_task,
        })
    }

    /// Panics if the background listener has completed
    pub fn subscribe(&self, tenant_id: Uuid, lens_uid: Uid) -> LensUpdateReceiver {
        {
            // Can't recovery from poison error, so just panic
            if self.listener_task.is_finished() {
                tracing::error!(
                    message="Background task has failed, must be restarted",
                );
                self.subscriptions.clear();
                panic!("Background task has failed, must be restarted");
            }
        }

        let lens_uid = lens_uid.as_i64();
        match self.subscriptions.entry((tenant_id, lens_uid)) {
            Entry::Occupied(tx) => {
                tx.get().subscribe()
            },
            Entry::Vacant(vacancy) => {
                let (tx, rx) = broadcast_channel(1_000);
                vacancy.insert(tx);
                rx
            }
        }
    }
}

#[tracing::instrument]
fn listener_loop(
    mut listener: PgListener,
    subscriptions: Subscriptions,
) -> JoinHandle<()> {
    tokio::task::spawn(async move {
        loop {
            let update = match listener.try_recv().await {
                Ok(Some(update)) => update,
                Ok(None) => {
                    // Set "force re-establish flag,
                    continue;
                }
                Err(e) => {
                    tracing::error!(
                                message="Failed to receive next update",
                                error=?e
                            );
                    continue;
                }
            };

            let update: LensUpdateRow = match serde_json::from_str(update.payload()) {
                Ok(update) => update,
                Err(e) => {
                    tracing::error!(
                                message="Failed to deserialize update row",
                                error=?e
                            );
                    continue;
                }
            };

            let tx = match subscriptions.get(&(update.tenant_id, update.lens_uid)) {
                Some(tx) => tx,
                // No one cares about these updates
                None => continue,
            };

            let lens_update = match LensUpdate::deserialize(&update.lens_update[..]) {
                Ok(update) => update,
                Err(e) => {
                    tracing::error!(
                        message="Failed to deserialize update protobuf",
                        error=?e
                    );
                    continue;
                }
            };
            if let Err(e) = tx.send((lens_update, update.update_offset)) {
                tracing::debug!(
                        message="All receivers closed, dropping message",
                        error=?e,
                    );
            };
        }
    }.instrument(tracing::info_span!("listener_task")))
}