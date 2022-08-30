use std::{
    collections::HashMap,
    convert::TryFrom,
};

use blake2::{
    digest::consts::U16,
    Blake2b,
    Digest,
};
use failure::Error;
use rusoto_dynamodb::AttributeValue;
use rust_proto::graplinc::grapl::common::v1beta1::types::Uid;
use serde::{
    Deserialize,
    Serialize,
};

type Blake2b16 = Blake2b<U16>;

#[derive(Debug)]
pub struct UnidSession {
    pub pseudo_key: String,
    pub node_type: String,
    pub timestamp: u64,
    pub is_creation: bool, // Is this a creation event
}

impl UnidSession {
    pub fn new(
        tenant_id: uuid::Uuid,
        node_type: String,
        pseudo_key: &str,
        timestamp: u64,
        is_creation: bool,
    ) -> Self {
        let mut hasher = Blake2b16::new();
        hasher.update(tenant_id.as_bytes());
        hasher.update(node_type.as_bytes());
        hasher.update(pseudo_key.as_bytes());
        let pseudo_key = hex::encode(hasher.finalize());
        Self {
            pseudo_key,
            node_type,
            timestamp,
            is_creation,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: u64,
    pub pseudo_key: String,
    pub create_time: u64,
    pub end_time: u64,
    pub is_create_canon: bool,
    pub is_end_canon: bool,
    pub version: u64, // This is an atomic version used for transactions
}

impl TryFrom<HashMap<String, AttributeValue>> for Session {
    type Error = Error;
    fn try_from(map: HashMap<String, AttributeValue>) -> Result<Self, Error> {
        Ok(serde_dynamodb::from_hashmap(map)?)
    }
}

pub(crate) fn shave_int(input: u64, digits: u8) -> u64 {
    let digits = 10u64.pow((digits as u32) + 1u32);
    input - (input % digits)
}
