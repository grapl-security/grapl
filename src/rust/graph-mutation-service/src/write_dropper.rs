use std::future::Future;

use dashmap::mapref::entry::Entry;
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct EdgeKey {
    tenant_id: uuid::Uuid,
    source_uid: Uid,
    dest_uid: Uid,
    edge_name: EdgeName,
}

#[derive(Default)]
pub struct WriteDropper {
    max_i64: dashmap::DashMap<PropertyKey, i64>,
    min_i64: dashmap::DashMap<PropertyKey, i64>,
    imm_i64: dashmap::DashSet<PropertyKey>,
    max_u64: dashmap::DashMap<PropertyKey, u64>,
    min_u64: dashmap::DashMap<PropertyKey, u64>,
    imm_u64: dashmap::DashSet<PropertyKey>,
    imm_string: dashmap::DashSet<PropertyKey>,
    node_type: dashmap::DashSet<NodeTypeKey>,
    edges: dashmap::DashSet<EdgeKey>,
}

impl WriteDropper {
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
                entry.insert(value);
            }
            Entry::Occupied(mut entry) => {
                if value > *entry.get() {
                    callback().await?;
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
                entry.insert(value);
            }
            Entry::Occupied(mut entry) => {
                if value < *entry.get() {
                    callback().await?;
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
                entry.insert(value);
            }
            Entry::Occupied(mut entry) => {
                if value > *entry.get() {
                    callback().await?;
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
                entry.insert(value);
            }
            Entry::Occupied(mut entry) => {
                if value < *entry.get() {
                    callback().await?;
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
        callback: impl FnOnce() -> Fut,
    ) -> Result<(), E>
    where
        Fut: Future<Output = Result<T, E>>,
    {
        let fkey = EdgeKey {
            tenant_id,
            source_uid,
            dest_uid,
            edge_name: f_edge_name,
        };
        let rkey = EdgeKey {
            tenant_id,
            dest_uid,
            source_uid,
            edge_name: r_edge_name,
        };
        if !self.edges.contains(&fkey) || !self.edges.contains(&rkey) {
            callback().await?;
            self.edges.insert(fkey);
            self.edges.insert(rkey);
        }
        Ok(())
    }
}
