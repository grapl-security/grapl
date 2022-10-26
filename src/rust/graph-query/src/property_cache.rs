use std::{
    fmt::Debug,
    hash::Hash,
};

use moka::future::{
    Cache,
    CacheBuilder,
};
use rust_proto::graplinc::grapl::common::v1beta1::types::Uid;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CacheMatch {
    Matched,
    NotMatched,
    CouldMatch,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    property_name: String,
    uid: Uid,
    tenant_id: uuid::Uuid,
}

#[derive(Clone)]
pub struct PropertyCache {
    string_cache: Cache<Key, String>,
    int_cache: Cache<Key, i64>,
    uint_cache: Cache<Key, u64>,
}

impl PropertyCache {
    pub fn new(
        string_cache: Cache<Key, String>,
        int_cache: Cache<Key, i64>,
        uint_cache: Cache<Key, u64>,
    ) -> Self {
        Self {
            string_cache,
            int_cache,
            uint_cache,
        }
    }
}
