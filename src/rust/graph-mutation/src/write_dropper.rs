use std::{
    future::Future,
    hash::Hash,
};

use moka::future::Cache;
use rust_proto::graplinc::grapl::common::v1beta1::types::{
    EdgeName,
    NodeType,
    PropertyName,
    Uid,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct NodeTypeKey {
    tenant_id: uuid::Uuid,
    uid: Uid,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PropertyKey {
    tenant_id: uuid::Uuid,
    node_type: NodeType,
    property_name: PropertyName,
}

// This enum/return type mostly exists for testing behavior.
#[derive(PartialEq, Eq)]
pub enum WriteDropStatus {
    Stored,
    Dropped,
}

/// WriteDropper lets us save database IO by proactively "dropping" db writes
/// that wouldn't change eventual outcome.
/// EXAMPLE 1: Immutable String
///   An immutable string can't change, so if we receive two instructions to
///   write an immutable string, we only have to write one of them.
/// EXAMPLE 2: Max u64/i64 (aka IncrOnly)
///   If you have a property that can only increment, and we've previously
///   written 5 to the DB, there's no reason to write a 4 if we encounter it.
#[allow(dead_code)] // TODO https://github.com/grapl-security/issue-tracker/issues/1028
#[derive(Clone, Debug)]
pub struct WriteDropper {
    max_i64: Cache<PropertyKey, i64>,
    min_i64: Cache<PropertyKey, i64>,
    imm_i64: Cache<PropertyKey, ()>,
    max_u64: Cache<PropertyKey, u64>,
    min_u64: Cache<PropertyKey, u64>,
    imm_u64: Cache<PropertyKey, ()>,
    imm_string: Cache<PropertyKey, ()>,
    node_type: Cache<NodeTypeKey, ()>,
    edges: Cache<[u8; 16], ()>,
}

impl WriteDropper {
    pub fn new(max_size: u64) -> Self {
        Self {
            max_i64: Cache::new(max_size),
            min_i64: Cache::new(max_size),
            imm_i64: Cache::new(max_size),
            max_u64: Cache::new(max_size),
            min_u64: Cache::new(max_size),
            imm_u64: Cache::new(max_size),
            imm_string: Cache::new(max_size),
            node_type: Cache::new(max_size),
            edges: Cache::new(max_size),
        }
    }

    #[tracing::instrument(skip(self, callback), err)]
    pub async fn check_max_i64<T, E, Fut>(
        &self,
        tenant_id: uuid::Uuid,
        node_type: NodeType,
        property_name: PropertyName,
        value: i64,
        callback: impl FnOnce() -> Fut,
    ) -> Result<WriteDropStatus, E>
    where
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        let key = PropertyKey {
            tenant_id,
            node_type,
            property_name,
        };
        get_or_insert_into_cache(self.max_i64.clone(), key, value, callback, |new, old| {
            new > old
        })
        .await
    }

    #[tracing::instrument(skip(self, callback), err)]
    pub async fn check_min_i64<T, E, Fut>(
        &self,
        tenant_id: uuid::Uuid,
        node_type: NodeType,
        property_name: PropertyName,
        value: i64,
        callback: impl FnOnce() -> Fut,
    ) -> Result<WriteDropStatus, E>
    where
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        let key = PropertyKey {
            tenant_id,
            node_type,
            property_name,
        };
        get_or_insert_into_cache(self.min_i64.clone(), key, value, callback, |new, old| {
            new < old
        })
        .await
    }
    #[tracing::instrument(skip(self, callback), err)]
    pub async fn check_imm_i64<T, E, Fut>(
        &self,
        tenant_id: uuid::Uuid,
        node_type: NodeType,
        property_name: PropertyName,
        callback: impl FnOnce() -> Fut,
    ) -> Result<WriteDropStatus, E>
    where
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        let key = PropertyKey {
            tenant_id,
            node_type,
            property_name,
        };
        get_or_insert_into_cache(self.imm_i64.clone(), key, (), callback, |_, _| false).await
    }

    #[tracing::instrument(skip(self, callback), err)]
    pub async fn check_max_u64<T, E, Fut>(
        &self,
        tenant_id: uuid::Uuid,
        node_type: NodeType,
        property_name: PropertyName,
        value: u64,
        callback: impl FnOnce() -> Fut,
    ) -> Result<WriteDropStatus, E>
    where
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        let key = PropertyKey {
            tenant_id,
            node_type,
            property_name,
        };
        get_or_insert_into_cache(self.max_u64.clone(), key, value, callback, |new, old| {
            new > old
        })
        .await
    }

    #[tracing::instrument(skip(self, callback), err)]
    pub async fn check_min_u64<T, E, Fut>(
        &self,
        tenant_id: uuid::Uuid,
        node_type: NodeType,
        property_name: PropertyName,
        value: u64,
        callback: impl FnOnce() -> Fut,
    ) -> Result<WriteDropStatus, E>
    where
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        let key = PropertyKey {
            tenant_id,
            node_type,
            property_name,
        };
        get_or_insert_into_cache(self.min_u64.clone(), key, value, callback, |new, old| {
            new < old
        })
        .await
    }

    #[tracing::instrument(skip(self, callback), err)]
    pub async fn check_imm_u64<T, E, Fut>(
        &self,
        tenant_id: uuid::Uuid,
        node_type: NodeType,
        property_name: PropertyName,
        callback: impl FnOnce() -> Fut,
    ) -> Result<WriteDropStatus, E>
    where
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        let key = PropertyKey {
            tenant_id,
            node_type,
            property_name,
        };
        get_or_insert_into_cache(self.imm_u64.clone(), key, (), callback, |_, _| false).await
    }

    #[tracing::instrument(skip(self, callback), err)]
    pub async fn check_imm_string<T, E, Fut>(
        &self,
        tenant_id: uuid::Uuid,
        node_type: NodeType,
        property_name: PropertyName,
        callback: impl FnOnce() -> Fut,
    ) -> Result<WriteDropStatus, E>
    where
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        let key = PropertyKey {
            tenant_id,
            node_type,
            property_name,
        };
        get_or_insert_into_cache(self.imm_string.clone(), key, (), callback, |_, _| false).await
    }

    #[tracing::instrument(skip(self, callback), err)]
    pub async fn check_node_type<T, E, Fut>(
        &self,
        tenant_id: uuid::Uuid,
        uid: Uid,
        callback: impl FnOnce() -> Fut,
    ) -> Result<WriteDropStatus, E>
    where
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        let key = NodeTypeKey { tenant_id, uid };
        get_or_insert_into_cache(self.node_type.clone(), key, (), callback, |_, _| false).await
    }

    #[tracing::instrument(skip(self, callback), err)]
    pub async fn check_edges<T, E, Fut>(
        &self,
        tenant_id: uuid::Uuid,
        source_uid: Uid,
        dest_uid: Uid,
        f_edge_name: EdgeName,
        r_edge_name: EdgeName,
        callback: impl FnOnce(EdgeName, EdgeName) -> Fut,
    ) -> Result<(), E>
    where
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        tracing::debug!(message = "Performing insert",);
        callback(f_edge_name, r_edge_name).await?;
        tracing::debug!(message = "Insert performed");

        // TODO: Resurrect the below, delete the above
        // https://github.com/grapl-security/issue-tracker/issues/1028

        // let fkey = edge_key(tenant_id, source_uid, dest_uid, &f_edge_name);
        //
        // // We always insert both the forward and reverse edges in a batch insert
        // if !self.edges.contains(&fkey) {
        //     handle_full!(self, edges);
        //     let rkey = edge_key(tenant_id, dest_uid, source_uid, &r_edge_name);
        //
        //     callback(f_edge_name, r_edge_name).await?;
        //
        //     self.edges.insert(fkey);
        //     self.edges.insert(rkey);
        // }
        Ok(())
    }
}

async fn get_or_insert_into_cache<Key, Value, T, E, Fut>(
    cache: Cache<Key, Value>,
    key: Key,
    new_value: Value,
    callback: impl FnOnce() -> Fut,
    is_new_value_better: impl FnOnce(&Value, &Value) -> bool,
) -> Result<WriteDropStatus, E>
where
    Key: Hash + Eq + Clone + Sync + Send + 'static,
    Value: Clone + Sync + Send + 'static,
    Fut: Future<Output = Result<T, E>>,
    E: std::error::Error,
{
    let should_insert_value = match cache.get(&key) {
        None => true,
        Some(stored_value) => is_new_value_better(&new_value, &stored_value),
    };

    if should_insert_value {
        callback().await?;
        cache.insert(key, new_value.clone()).await;
    }
    Ok(match should_insert_value {
        true => WriteDropStatus::Stored,
        false => WriteDropStatus::Dropped,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[derive(Debug, thiserror::Error)]
    enum CallbackError {}

    #[tokio::test]
    async fn test_every_class_of_cache() -> eyre::Result<()> {
        let tenant_id = uuid::Uuid::new_v4();
        let node_type = NodeType {
            value: "arbitrary_node_type".to_string(),
        };
        let property_name = PropertyName {
            value: "arbitrary_prop_name".to_string(),
        };

        let callback = || async {
            let res: Result<(), CallbackError> = Ok(());
            res
        };
        let write_dropper = Arc::new(WriteDropper::new(3));

        // ##### check_max_i64 #####
        {
            let write_dropper = Arc::clone(&write_dropper);
            let nt = node_type.clone();
            let pn = property_name.clone();
            let check = move |value| {
                let write_dropper = Arc::clone(&write_dropper);
                let nt = nt.clone();
                let pn = pn.clone();
                async move {
                    write_dropper
                        .check_max_i64(tenant_id, nt, pn, value, callback)
                        .await
                }
            };

            let status = check(3).await?;
            eyre::ensure!(status == WriteDropStatus::Stored, "initial always stores");
            let status = check(3).await?;
            eyre::ensure!(status == WriteDropStatus::Dropped, "same value, drop it");
            let status = check(-3).await?;
            eyre::ensure!(status == WriteDropStatus::Dropped, "lesser value, drop it");
            let status = check(4).await?;
            eyre::ensure!(status == WriteDropStatus::Stored, "greater value, store it");
        }

        // ##### check_min_i64 #####
        {
            let write_dropper = Arc::clone(&write_dropper);
            let nt = node_type.clone();
            let pn = property_name.clone();
            let check = move |value| {
                let write_dropper = Arc::clone(&write_dropper);
                let nt = nt.clone();
                let pn = pn.clone();
                async move {
                    write_dropper
                        .check_min_i64(tenant_id, nt, pn, value, callback)
                        .await
                }
            };

            let status = check(3).await?;
            eyre::ensure!(status == WriteDropStatus::Stored, "initial always stores");
            let status = check(3).await?;
            eyre::ensure!(status == WriteDropStatus::Dropped, "same value, drop it");
            let status = check(4).await?;
            eyre::ensure!(status == WriteDropStatus::Dropped, "greater value, drop it");
            let status = check(-3).await?;
            eyre::ensure!(status == WriteDropStatus::Stored, "lesser value, store it");
        }

        // ##### check_imm_i64 #####
        {
            let write_dropper = Arc::clone(&write_dropper);
            let nt = node_type.clone();
            let pn = property_name.clone();
            let check = move || {
                let write_dropper = Arc::clone(&write_dropper);
                let nt = nt.clone();
                let pn = pn.clone();
                async move {
                    write_dropper
                        .check_imm_i64(tenant_id, nt, pn, callback)
                        .await
                }
            };

            let status = check().await?;
            eyre::ensure!(status == WriteDropStatus::Stored, "initial always stores");
            let status = check().await?;
            eyre::ensure!(
                status == WriteDropStatus::Dropped,
                "immutable"
            );
        }

        // ##### check_max_u64 #####
        {
            let write_dropper = Arc::clone(&write_dropper);
            let nt = node_type.clone();
            let pn = property_name.clone();
            let check = move |value| {
                let write_dropper = Arc::clone(&write_dropper);
                let nt = nt.clone();
                let pn = pn.clone();
                async move {
                    write_dropper
                        .check_max_u64(tenant_id, nt, pn, value, callback)
                        .await
                }
            };

            let status = check(3).await?;
            eyre::ensure!(status == WriteDropStatus::Stored, "initial always stores");
            let status = check(3).await?;
            eyre::ensure!(status == WriteDropStatus::Dropped, "same value, drop it");
            let status = check(2).await?;
            eyre::ensure!(status == WriteDropStatus::Dropped, "lesser value, drop it");
            let status = check(4).await?;
            eyre::ensure!(status == WriteDropStatus::Stored, "greater value, store it");
        }

        // ##### check_min_u64 #####
        {
            let write_dropper = Arc::clone(&write_dropper);
            let nt = node_type.clone();
            let pn = property_name.clone();
            let check = move |value| {
                let write_dropper = Arc::clone(&write_dropper);
                let nt = nt.clone();
                let pn = pn.clone();
                async move {
                    write_dropper
                        .check_min_u64(tenant_id, nt, pn, value, callback)
                        .await
                }
            };

            let status = check(3).await?;
            eyre::ensure!(status == WriteDropStatus::Stored, "initial always stores");
            let status = check(3).await?;
            eyre::ensure!(status == WriteDropStatus::Dropped, "same value, drop it");
            let status = check(4).await?;
            eyre::ensure!(status == WriteDropStatus::Dropped, "greater value, drop it");
            let status = check(2).await?;
            eyre::ensure!(status == WriteDropStatus::Stored, "lesser value, store it");
        }

        // ##### check_imm_u64 #####
        {
            let write_dropper = Arc::clone(&write_dropper);
            let nt = node_type.clone();
            let pn = property_name.clone();
            let check = move || {
                let write_dropper = Arc::clone(&write_dropper);
                let nt = nt.clone();
                let pn = pn.clone();
                async move {
                    write_dropper
                        .check_imm_u64(tenant_id, nt, pn, callback)
                        .await
                }
            };

            let status = check().await?;
            eyre::ensure!(status == WriteDropStatus::Stored, "initial always stores");
            let status = check().await?;
            eyre::ensure!(
                status == WriteDropStatus::Dropped,
                "immutable"
            );
        }

        // ##### check_imm_string #####
        {
            let write_dropper = Arc::clone(&write_dropper);
            let nt = node_type.clone();
            let pn = property_name.clone();
            let check = move || {
                let write_dropper = Arc::clone(&write_dropper);
                let nt = nt.clone();
                let pn = pn.clone();
                async move {
                    write_dropper
                        .check_imm_string(tenant_id, nt, pn, callback)
                        .await
                }
            };

            let status = check().await?;
            eyre::ensure!(status == WriteDropStatus::Stored, "initial always stores");
            let status = check().await?;
            eyre::ensure!(
                status == WriteDropStatus::Dropped,
                "immutable"
            );
        }

        // ##### check_node_type #####
        {
            let write_dropper = Arc::clone(&write_dropper);
            let uid = Uid::from_u64(123).unwrap();

            let status = write_dropper.check_node_type(tenant_id, uid, callback).await?;
            eyre::ensure!(status == WriteDropStatus::Stored, "initial always stores");
            let status = write_dropper.check_node_type(tenant_id, uid, callback).await?;
            eyre::ensure!(status == WriteDropStatus::Dropped, "immutable");
        }

        Ok(())
    }
}
