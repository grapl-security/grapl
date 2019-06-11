use failure::Error;
use rusoto_dynamodb::{AttributeValue, DynamoDb, PutItemInput, QueryInput};
use serde_dynamodb::ToQueryInput;
use sessions::shave_int;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheKey<'a> {
    pseudo_key: &'a str,
    ttl_ts: u64,
}

// The node-identifier incurs a lot of retries, due to the way it performs guessing
pub struct RetrySessionCache<D>
where
    D: DynamoDb,
{
    table_name: String,
    ttl_ts: u64,
    dynamo: D,
}

impl<D> RetrySessionCache<D>
where
    D: DynamoDb,
{
    pub fn new(table_name: impl Into<String>, dynamo: D) -> Self {
        let ttl_ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 86400;
        Self {
            table_name: table_name.into(),
            ttl_ts,
            dynamo,
        }
    }

    pub fn in_cache(&self, old_key: impl Into<String>) -> Result<bool, Error> {
        let query = QueryInput {
            limit: Some(1),
            table_name: self.table_name.clone(),
            key_condition_expression: Some("pseudo_key = :pseudo_key".into()),
            expression_attribute_values: Some(hmap! {
                ":pseudo_key".to_owned() => AttributeValue {
                    s: Some(old_key.into()),
                    ..Default::default()
                }
            }),
            ..Default::default()
        };

        let res = wait_on!(self.dynamo.query(query))?;

        if let Some(items) = res.items {
            match &items[..] {
                [item] => Ok(true),
                _ => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    pub fn put_cache(&self, old_key: impl AsRef<str>) -> Result<(), Error> {
        let pseudo_key = CacheKey {
            pseudo_key: old_key.as_ref(),
            ttl_ts: self.ttl_ts,
        };

        let put_req = PutItemInput {
            item: serde_dynamodb::to_hashmap(&pseudo_key).unwrap(),
            table_name: self.table_name.clone(),
            ..Default::default()
        };

        wait_on!(self.dynamo.put_item(put_req))?;

        Ok(())
    }
}
