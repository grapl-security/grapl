#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;

extern crate serde;
extern crate serde_json;
extern crate dgraph_client;
extern crate graph_descriptions;

use failure::Error;
use graph_descriptions::graph_description::*;

use dgraph_client::api_grpc::DgraphClient;
use std::time::SystemTime;
use std::collections::HashMap;


#[derive(Serialize, Deserialize, Debug)]
struct Uid {
    uid: String
}

#[derive(Serialize, Deserialize, Debug)]
struct DgraphResponse {
    // TODO: Custom hasher for qN key format
    // TODO: SmallVec<1>
    response: HashMap<String, Vec<Uid>>,
}

struct DgraphQuery {
    // TODO: Cache the string version of the key using a short string of size 4
    key: u16,
    query: String,
    insert: serde_json::Value,
}

impl DgraphQuery {
    fn get_key(&self) -> u16 {
        self.key
    }

    fn mut_insert(&mut self) -> &mut serde_json::Value {
        &mut self.insert
    }


    fn get_insert(&mut self) -> & serde_json::Value {
        &self.insert
    }
}


impl<'a> From<(usize, &'a NodeDescription)> for DgraphQuery {
    fn from((key, node): (usize, &NodeDescription)) -> DgraphQuery {
        let key = key as u16;
        let node_key = node.get_key();
        let query = format!(r#"
            {{
                q{key}(func: eq(node_key, "{node_key}"))
                {{
                    uid,
                }}
            }}"#, key=key, node_key=node_key);


        let mut insert = (*node).clone().into_json();
        insert.as_object_mut().unwrap().remove("node_key");

        DgraphQuery {
            key,
            query,
            insert
        }
    }
}

#[derive(Clone, Debug)]
enum UpsertStatus {
    Complete,
    Incomplete
}

struct QueryManager {
    queries: Vec<DgraphQuery>,
    client: DgraphClient,
    query_buffer: String,
    insert_buffer: String,
}

impl QueryManager {
    pub fn new(client: DgraphClient, nodes: &[NodeDescription]) -> QueryManager {
        let queries: Vec<_> = nodes.iter().enumerate().map(DgraphQuery::from).collect();

        let buf_len: usize = queries.iter().map(|q| &q.query).map(String::len).sum();
        let buf_len = buf_len + queries.len();
        // Preallocate buffer + 3 for the wrapping characters
        let mut query_buffer= String::with_capacity(buf_len + 3);
        let mut insert_buffer= String::with_capacity(buf_len + 3);

        QueryManager {
            queries,
            client,
            query_buffer,
            insert_buffer
        }
    }

    pub fn upsert_all(&mut self) -> Result<UpsertStatus, Error> {
        let mut txn = dgraph_client::api::TxnContext::new();
        txn.set_start_ts(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs());

        // clear, and then fill, the internal query buffer
        self.generate_queries();

        // Query dgraph for remaining nodes
        let query_response = self.query_all()?;

        // Generate upserts
        self.generate_insert(query_response);


        // perform upsert
        let mut mutation = dgraph_client::api::Mutation::new();
        mutation.commit_now = true;
        mutation.set_json = self.insert_buffer.as_bytes().to_owned();
        let mut_res = self.client.mutate(&mutation)?;

        txn.set_commit_ts(mut_res.context.get_ref().commit_ts);
        let txn_ctx = self.client.commit_or_abort(&txn)?;
        txn_ctx.aborted;

        unimplemented!()
    }

    // TODO: Cache node-> id mappings
    fn generate_queries(&mut self) {
        self.query_buffer.clear();
        let all_queries = &mut self.query_buffer;

        all_queries.push_str("{");
        self.queries.iter().for_each(|query| {
            all_queries.push_str(&query.query);
            all_queries.push_str(",");
        });
        all_queries.push_str("}");

    }

    fn query_all(&mut self) -> Result<DgraphResponse, Error> {
        let mut req = dgraph_client::api::Request::new();

        req.query = self.query_buffer.to_string();

        let resp = self.client.query(&req).expect("upsert query");
        Ok(serde_json::from_slice(resp.get_json())?)
    }

    fn generate_insert(&mut self, response: DgraphResponse) -> Result<(), Error> {
        // Note that we can not cache these strings because the query response may have changed
        // and that will impact the merging logic

        // Actually - I believe we can cache them. What we've received so far is a mapping of our
        // query_key to uids
        // That mappind should never change.
        //

        // TODO: Reuse this vector
        let insert_buffer = &mut self.insert_buffer;
        insert_buffer.push_str("{");
        for to_insert in &mut self.queries {
            let mut q_key = String::from("q");
            // TODO: Just have the key be a string to begin with
            q_key.push_str(&to_insert.key.to_string());
            let q_key = q_key;

            let response = response.response.get(&q_key);
            let uid = match response {
                Some(v) => &v[0].uid,
                // If we generate a query we should never *not* have it in a response
                None => bail!("Could not find response with key: {}", &q_key)
            };
            to_insert.mut_insert()["uid"] = serde_json::Value::from(uid.clone());
            insert_buffer.push_str(&to_insert.get_insert().to_string());
            insert_buffer.push_str(",");
        }

        insert_buffer.push_str("}");

        unimplemented!()
    }
}

fn main() {
    println!("Hello, world!");
}
