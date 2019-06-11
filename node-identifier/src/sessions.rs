#![allow(dead_code, unused_variables)]
use std::collections::HashMap;

use base58::ToBase58;
use failure::Error;
use futures::future::Future;
use rusoto_core::Region;
use rusoto_dynamodb::{
    AttributeValue, Condition, DeleteItemInput, DynamoDb, DynamoDbClient, GetItemInput,
    ListTablesInput, PutItemInput, QueryInput, Update, UpdateItemInput,
};
use sha2::{Digest, Sha256};
use std::convert::TryFrom;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct UnidSession {
    pub pseudo_key: String,
    pub timestamp: u64,
    pub is_creation: bool, // Is this a creation event
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub pseudo_key: String, // hostname-pid
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

pub fn shave_int(input: u64, digits: u8) -> u64 {
    let digits = 10u64.pow((digits as u32) + 1u32);
    (input - (input % digits))
}
