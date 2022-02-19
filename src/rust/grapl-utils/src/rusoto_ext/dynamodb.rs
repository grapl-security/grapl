use std::collections::HashMap;

use async_trait::async_trait;
use rusoto_core::RusotoError;
use rusoto_dynamodb::{
    AttributeValue,
    BatchGetItemError,
    BatchGetItemInput,
    BatchGetItemOutput,
    DynamoDb,
    KeysAndAttributes,
};

const DYNAMODB_MAX_BATCH_GET_ITEM_SIZE: usize = 100;

#[async_trait]
pub trait GraplDynamoDbClientExt: DynamoDb + Send + Sync {
    /**
    The original `batch_get_item` method imposes some restrictions on querying. Namely:
    * You must request 100 or fewer items otherwise an error is returned
    * If the response would be over 16MB, only up-to 16MB of requested data is returned. A list of unprocessed items is also returned

    This method attempts to solve this problem relatively generally.

    You may pass a BatchGetItemInput with *more than* 100 items requested and `batch_get_item_reliably`
    guarantees that all of the items will be requested within DynamoDb's rules on number of items requested and
    size of response. This works across table_names as well, therefore 75 items from one table and 50 from another
    will still be requested within DynamoDb's rules and remain compliant.
     */
    async fn batch_get_item_reliably(
        &self,
        batch_get_items_input: BatchGetItemInput,
    ) -> Result<BatchGetItemOutput, RusotoError<BatchGetItemError>> {
        /*
           1. Create a hashmap that stores KeysAndAttributes per table (without 'keys')
           2. Convert the requested items into an iterator of (table_name, HashMap<String, AttributeValue>)
           3. While this list is not empty
               1. Grab up to 100 items
               2. Sort AttributeValue's based on table_name
               3. Per Vec<AttributeValue> assemble a KeysAndAttributes per table_name based on the KeysAndAttribute shells captured earlier
               4. Assemble a BatchGetItem operation
               5. Request
               6. record results
               7. capture all unprocessed keys and push them back into the queue of unprocessed items
               8. repeat
           4. Final formatting and return
        */
        // holds the properties for `KeysAndAttributes` used for a particular table minus the requested items (we'll assemble this later)
        let mut key_and_attribute_shells = HashMap::new();

        // extract all of the requested rows from all of the tables into a vec
        // additionally, keep a copy of the `KeysAndAttributes` for each table (with the items removed) so we can copy the properties later
        let mut pending_items: Vec<(String, HashMap<String, AttributeValue>)> =
            batch_get_items_input
                .request_items
                .into_iter()
                .flat_map(|(table, mut keys_and_attributes)| {
                    let request_items = keys_and_attributes
                        .keys
                        .drain(0..keys_and_attributes.keys.len())
                        .map(|row_properties| (table.clone(), row_properties))
                        .collect::<Vec<(String, HashMap<String, AttributeValue>)>>();

                    key_and_attribute_shells.insert(table, keys_and_attributes);

                    request_items
                })
                .collect();

        let mut total_consumed_capacity = Vec::new();
        let mut total_responses = HashMap::<String, Vec<HashMap<String, AttributeValue>>>::new();

        // main loop for processing
        // when this is empty, we've finished processing our data
        while !pending_items.is_empty() {
            // type not needed due to inference, but added anyway to make IDEs everywhere happier :)
            let mut requested_items_for_table: HashMap<
                String,
                Vec<HashMap<String, AttributeValue>>,
            > = HashMap::new();

            // grab up to 100 items from the list of items to request and put them into a hashmap that maps
            // <table_name, list of items to request for that table>
            pending_items
                .drain(0..std::cmp::min(pending_items.len(), DYNAMODB_MAX_BATCH_GET_ITEM_SIZE))
                .for_each(|(table_name, row_keys)| {
                    let entry = requested_items_for_table
                        .entry(table_name)
                        .or_insert(Vec::new());

                    entry.push(row_keys);
                });

            // convert the hashmap mapping table_name and items to request for that table into actual KeysAndAttributes
            // based on the copies we kept earlier
            let manufacted_request_items = requested_items_for_table
                .into_iter()
                .map(|(table_name, pending_items)| {
                    // grab the previously kept KeysAndAttributes to copy properties from
                    let keys_and_attributes_template =
                        match key_and_attribute_shells.get(&table_name) {
                            Some(keys_and_attributes) => keys_and_attributes.clone(),
                            // Unable to find our previous copy? Just make a new one with some defaults
                            None => Default::default(),
                        };

                    let manufacted_keys_and_attributes = KeysAndAttributes {
                        keys: pending_items,
                        ..keys_and_attributes_template
                    };

                    (table_name, manufacted_keys_and_attributes)
                })
                .collect();

            let batch_request = BatchGetItemInput {
                request_items: manufacted_request_items,
                return_consumed_capacity: batch_get_items_input.return_consumed_capacity.clone(),
            };

            // We've manufactured a compliant request!
            // Let's issue the request to DynamoDb to fetch results and handle potentially unprocessed keys
            let response = self.batch_get_item(batch_request).await?;

            // record consumed capacity, if enabled
            if let Some(capacity) = response.consumed_capacity {
                total_consumed_capacity.extend(capacity);
            }

            if let Some(response_map) = response.responses {
                // for each table in response, record the rows for that table
                response_map.into_iter().for_each(|(table_name, rows)| {
                    let table_rows = total_responses.entry(table_name).or_insert(Vec::new());
                    table_rows.extend(rows);
                })
            }

            // if a response was too large, we may have unprocessed keys.
            // we can just directly insert these back into pending items for processing
            if let Some(unprocessed_keys) = response.unprocessed_keys {
                let unprocessed_keys: Vec<_> = unprocessed_keys
                    .into_iter()
                    .flat_map(|(table_name, keys_and_attributes)| {
                        keys_and_attributes
                            .keys
                            .into_iter()
                            .map(|row_keys| (table_name.clone(), row_keys))
                            .collect::<Vec<_>>()
                    })
                    .collect();
                pending_items.extend(unprocessed_keys);
            }
        }

        Ok(BatchGetItemOutput {
            consumed_capacity: batch_get_items_input
                .return_consumed_capacity
                .map(|_| total_consumed_capacity),
            responses: Some(total_responses),
            unprocessed_keys: None,
        })
    }
}

impl<I> GraplDynamoDbClientExt for I where I: DynamoDb + Send + Sync + Sized {}
