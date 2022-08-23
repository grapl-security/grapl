use std::future::Future;

use blake2::{
    digest::consts::U16,
    Blake2b,
    Digest,
};
use dashmap::mapref::entry::Entry;
use rust_proto::graplinc::grapl::common::v1beta1::types::{
    EdgeName,
    NodeType,
    PropertyName,
    Uid,
};

type Blake2b16 = Blake2b<U16>;

macro_rules! handle_full {
    ($self:ident, $cache:ident) => {{
        if $self.$cache.len() >= $self.max_size {
            // Do not 'inline' the `if let Some` or you may cause a deadlock
            let entry = $self.$cache.iter().next().map(|i| i.key().clone());
            if let Some(entry) = entry {
                $self.$cache.remove(&entry);
            }
        }
    }};
}

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

pub struct WriteDropper {
    max_i64: dashmap::DashMap<PropertyKey, i64>,
    min_i64: dashmap::DashMap<PropertyKey, i64>,
    imm_i64: dashmap::DashSet<PropertyKey>,
    max_u64: dashmap::DashMap<PropertyKey, u64>,
    min_u64: dashmap::DashMap<PropertyKey, u64>,
    imm_u64: dashmap::DashSet<PropertyKey>,
    imm_string: dashmap::DashSet<PropertyKey>,
    node_type: dashmap::DashSet<NodeTypeKey>,
    edges: dashmap::DashSet<[u8; 16], hash_hasher::HashBuildHasher>,
    max_size: usize,
}

impl WriteDropper {
    pub fn new(max_size: usize) -> Self {
        Self {
            max_i64: Default::default(),
            min_i64: Default::default(),
            imm_i64: Default::default(),
            max_u64: Default::default(),
            min_u64: Default::default(),
            imm_u64: Default::default(),
            imm_string: Default::default(),
            node_type: Default::default(),
            edges: Default::default(),
            max_size,
        }
    }

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
    {
        match self.max_i64.entry(PropertyKey {
            tenant_id,
            node_type,
            property_name,
        }) {
            Entry::Vacant(entry) => {
                callback().await?;
                handle_full!(self, max_i64);
                entry.insert(value);
            }
            Entry::Occupied(mut entry) => {
                if value > *entry.get() {
                    callback().await?;
                    handle_full!(self, max_i64);
                    entry.insert(value);
                }
            }
        }
        Ok(())
    }

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
    {
        match self.min_i64.entry(PropertyKey {
            tenant_id,
            node_type,
            property_name,
        }) {
            Entry::Vacant(entry) => {
                callback().await?;
                handle_full!(self, min_i64);
                entry.insert(value);
            }
            Entry::Occupied(mut entry) => {
                if value < *entry.get() {
                    callback().await?;
                    handle_full!(self, min_i64);
                    entry.insert(value);
                }
            }
        }
        Ok(())
    }
    pub async fn check_imm_i64<T, E, Fut>(
        &self,
        tenant_id: uuid::Uuid,
        node_type: NodeType,
        property_name: PropertyName,
        callback: impl FnOnce() -> Fut,
    ) -> Result<(), E>
    where
        Fut: Future<Output = Result<T, E>>,
    {
        let key = PropertyKey {
            tenant_id,
            node_type,
            property_name,
        };
        if !self.imm_i64.contains(&key) {
            callback().await?;
            handle_full!(self, imm_i64);
            self.imm_i64.insert(key);
        }
        Ok(())
    }

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
    {
        match self.max_u64.entry(PropertyKey {
            tenant_id,
            node_type,
            property_name,
        }) {
            Entry::Vacant(entry) => {
                callback().await?;
                handle_full!(self, max_u64);
                entry.insert(value);
            }
            Entry::Occupied(mut entry) => {
                if value > *entry.get() {
                    callback().await?;
                    handle_full!(self, max_u64);
                    entry.insert(value);
                }
            }
        }
        Ok(())
    }

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
    {
        match self.min_u64.entry(PropertyKey {
            tenant_id,
            node_type,
            property_name,
        }) {
            Entry::Vacant(entry) => {
                callback().await?;
                handle_full!(self, min_u64);
                entry.insert(value);
            }
            Entry::Occupied(mut entry) => {
                if value < *entry.get() {
                    callback().await?;
                    handle_full!(self, min_u64);
                    entry.insert(value);
                }
            }
        }
        Ok(())
    }
    pub async fn check_imm_u64<T, E, Fut>(
        &self,
        tenant_id: uuid::Uuid,
        node_type: NodeType,
        property_name: PropertyName,
        callback: impl FnOnce() -> Fut,
    ) -> Result<(), E>
    where
        Fut: Future<Output = Result<T, E>>,
    {
        let key = PropertyKey {
            tenant_id,
            node_type,
            property_name,
        };
        if !self.imm_u64.contains(&key) {
            callback().await?;
            handle_full!(self, imm_u64);
            self.imm_u64.insert(key);
        }
        Ok(())
    }

    pub async fn check_imm_string<T, E, Fut>(
        &self,
        tenant_id: uuid::Uuid,
        node_type: NodeType,
        property_name: PropertyName,
        callback: impl FnOnce() -> Fut,
    ) -> Result<(), E>
    where
        Fut: Future<Output = Result<T, E>>,
    {
        let key = PropertyKey {
            tenant_id,
            node_type,
            property_name,
        };
        if !self.imm_string.contains(&key) {
            callback().await?;
            handle_full!(self, imm_string);
            self.imm_string.insert(key);
        }
        Ok(())
    }

    pub async fn check_node_type<T, E, Fut>(
        &self,
        tenant_id: uuid::Uuid,
        uid: Uid,
        callback: impl FnOnce() -> Fut,
    ) -> Result<(), E>
    where
        Fut: Future<Output = Result<T, E>>,
    {
        let key = NodeTypeKey { tenant_id, uid };
        if !self.node_type.contains(&key) {
            callback().await?;
            handle_full!(self, node_type);
            self.node_type.insert(key);
        }
        Ok(())
    }

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
    {
        let fkey = edge_key(tenant_id, source_uid, dest_uid, &f_edge_name);

        // We always insert both the forward and reverse edges in a batch insert
        if !self.edges.contains(&fkey) {
            handle_full!(self, edges);
            let rkey = edge_key(tenant_id, dest_uid, source_uid, &r_edge_name);

            callback(f_edge_name, r_edge_name).await?;

            self.edges.insert(fkey);
            self.edges.insert(rkey);
        }
        Ok(())
    }
}
