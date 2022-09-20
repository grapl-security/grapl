use std::future::Future;

use blake2::{
    digest::consts::U16,
    Blake2b,
    Digest,
};
use dashmap::mapref::entry::Entry;
use moka::future::Cache;
use rust_proto::graplinc::grapl::common::v1beta1::types::{
    EdgeName,
    NodeType,
    PropertyName,
    Uid,
};

type Blake2b16 = Blake2b<U16>;

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

fn edge_key(
    tenant_id: uuid::Uuid,
    source_uid: Uid,
    dst_uid: Uid,
    edge_name: &EdgeName,
) -> [u8; 16] {
    let mut hasher = Blake2b16::new();
    hasher.update(tenant_id.as_bytes());
    hasher.update(source_uid.as_u64().to_le_bytes());
    hasher.update(dst_uid.as_u64().to_le_bytes());
    hasher.update(edge_name.value.as_bytes());
    hasher.update(b"my_input");
    hasher.finalize().into()
}

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
    ) -> Result<(), E>
    where
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error
    {

        tracing::debug!(message="Performing insert",);
        callback().await?;
        tracing::debug!(message="Insert performed");
        // match self.max_i64.entry(PropertyKey {
        //     tenant_id,
        //     node_type,
        //     property_name,
        // }) {
        //     Entry::Vacant(entry) => {
        //         callback().await?;
        //         handle_full!(self, max_i64);
        //         entry.insert(value);
        //     }
        //     Entry::Occupied(mut entry) => {
        //         if value > *entry.get() {
        //             callback().await?;
        //             handle_full!(self, max_i64);
        //             entry.insert(value);
        //         }
        //     }
        // }
        Ok(())
    }

    #[tracing::instrument(skip(self, callback), err)]
    pub async fn check_min_i64<T, E, Fut>(
        &self,
        tenant_id: uuid::Uuid,
        node_type: NodeType,
        property_name: PropertyName,
        value: i64,
        callback: impl FnOnce() -> Fut,
    ) -> Result<(), E>
    where
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error
    {

        tracing::debug!(message="Performing insert",);
        callback().await?;
        tracing::debug!(message="Insert performed");
        // match self.min_i64.entry(PropertyKey {
        //     tenant_id,
        //     node_type,
        //     property_name,
        // }) {
        //     Entry::Vacant(entry) => {
        //         callback().await?;
        //         handle_full!(self, min_i64);
        //         entry.insert(value);
        //     }
        //     Entry::Occupied(mut entry) => {
        //         if value < *entry.get() {
        //             callback().await?;
        //             handle_full!(self, min_i64);
        //             entry.insert(value);
        //         }
        //     }
        // }
        Ok(())
    }
    #[tracing::instrument(skip(self, callback), err)]
    pub async fn check_imm_i64<T, E, Fut>(
        &self,
        tenant_id: uuid::Uuid,
        node_type: NodeType,
        property_name: PropertyName,
        callback: impl FnOnce() -> Fut,
    ) -> Result<(), E>
    where
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error
    {
        tracing::debug!(message="Performing insert",);
        callback().await?;
        tracing::debug!(message="Insert performed");
        // let key = PropertyKey {
        //     tenant_id,
        //     node_type,
        //     property_name,
        // };
        // if !self.imm_i64.contains(&key) {
        //     callback().await?;
        //     handle_full!(self, imm_i64);
        //     self.imm_i64.insert(key);
        // }
        Ok(())
    }

    #[tracing::instrument(skip(self, callback), err)]
    pub async fn check_max_u64<T, E, Fut>(
        &self,
        tenant_id: uuid::Uuid,
        node_type: NodeType,
        property_name: PropertyName,
        value: u64,
        callback: impl FnOnce() -> Fut,
    ) -> Result<(), E>
    where
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error
    {

        tracing::debug!(message="Performing insert",);
        callback().await?;
        tracing::debug!(message="Insert performed");
        // match self.max_u64.entry(PropertyKey {
        //     tenant_id,
        //     node_type,
        //     property_name,
        // }) {
        //     Entry::Vacant(entry) => {
        //         callback().await?;
        //         handle_full!(self, max_u64);
        //         entry.insert(value);
        //     }
        //     Entry::Occupied(mut entry) => {
        //         if value > *entry.get() {
        //             callback().await?;
        //             handle_full!(self, max_u64);
        //             entry.insert(value);
        //         }
        //     }
        // }
        Ok(())
    }

    #[tracing::instrument(skip(self, callback), err)]
    pub async fn check_min_u64<T, E, Fut>(
        &self,
        tenant_id: uuid::Uuid,
        node_type: NodeType,
        property_name: PropertyName,
        value: u64,
        callback: impl FnOnce() -> Fut,
    ) -> Result<(), E>
    where
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error
    {

        tracing::debug!(message="Performing insert",);
        callback().await?;
        tracing::debug!(message="Insert performed");
        // match self.min_u64.entry(PropertyKey {
        //     tenant_id,
        //     node_type,
        //     property_name,
        // }) {
        //     Entry::Vacant(entry) => {
        //         callback().await?;
        //         handle_full!(self, min_u64);
        //         entry.insert(value);
        //     }
        //     Entry::Occupied(mut entry) => {
        //         if value < *entry.get() {
        //             callback().await?;
        //             handle_full!(self, min_u64);
        //             entry.insert(value);
        //         }
        //     }
        // }
        Ok(())
    }
    #[tracing::instrument(skip(self, callback), err)]
    pub async fn check_imm_u64<T, E, Fut>(
        &self,
        tenant_id: uuid::Uuid,
        node_type: NodeType,
        property_name: PropertyName,
        callback: impl FnOnce() -> Fut,
    ) -> Result<(), E>
    where
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error
    {

        tracing::debug!(message="Performing insert",);
        callback().await?;
        tracing::debug!(message="Insert performed");
        // let key = PropertyKey {
        //     tenant_id,
        //     node_type,
        //     property_name,
        // };
        // if !self.imm_u64.contains(&key) {
        //     callback().await?;
        //     handle_full!(self, imm_u64);
        //     self.imm_u64.insert(key);
        // }
        Ok(())
    }

    #[tracing::instrument(skip(self, callback), err)]
    pub async fn check_imm_string<T, E, Fut>(
        &self,
        tenant_id: uuid::Uuid,
        node_type: NodeType,
        property_name: PropertyName,
        callback: impl FnOnce() -> Fut,
    ) -> Result<(), E>
    where
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error
    {

        tracing::debug!(message="Performing insert",);
        callback().await?;
        tracing::debug!(message="Insert performed");
        // let key = PropertyKey {
        //     tenant_id,
        //     node_type,
        //     property_name,
        // };
        // if !self.imm_string.contains(&key) {
        //     callback().await?;
        //     handle_full!(self, imm_string);
        //     self.imm_string.insert(key);
        // }
        Ok(())
    }

    #[tracing::instrument(skip(self, callback), err)]
    pub async fn check_node_type<T, E, Fut>(
        &self,
        tenant_id: uuid::Uuid,
        uid: Uid,
        callback: impl FnOnce() -> Fut,
    ) -> Result<(), E>
    where
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error
    {

        tracing::debug!(message="Performing insert",);
        callback().await?;
        tracing::debug!(message="Insert performed");
        // let key = NodeTypeKey { tenant_id, uid };
        // if !self.node_type.contains(&key) {
        //     callback().await?;
        //     handle_full!(self, node_type);
        //     self.node_type.insert(key);
        // }
        Ok(())
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
        E: std::error::Error
    {

        tracing::debug!(message="Performing insert",);
        callback(f_edge_name, r_edge_name).await?;
        tracing::debug!(message="Insert performed");
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
