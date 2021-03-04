use crate::mutations::node_mutation::NodeUpsertGenerator;
use grapl_graph_descriptions::IdentifiedNode;
use dgraph_tonic::{Client as DgraphClient, Mutate, Mutation as DgraphMutation, MutationResponse};
use std::sync::Arc;
use futures_retry::{FutureRetry, RetryPolicy};
use grapl_graph_descriptions::Edge;

pub struct UpsertManager {
    pub dgraph_client: Arc<DgraphClient>,
    pub node_upsert_generator: NodeUpsertGenerator,
}

impl UpsertManager {
    pub async fn upsert_node(&mut self, node: &IdentifiedNode) -> u64 {
        let (creation_var_name, query, mutations) = self.node_upsert_generator.generate_upserts(
            0u128,
            0u128,
            node,
        );

        let combined_query = format!(r"
            {{
                {query}
            }}
        ", query=query);

        let dgraph_client = self.dgraph_client.clone();
        let mutations = mutations.to_vec();
        let res = enforce_transaction(move || {
            let mut txn = dgraph_client.new_mutated_txn();
            txn.upsert_and_commit_now(combined_query.clone(), mutations.clone())
        }).await;

        extract_uid(&creation_var_name, &res)
    }

    pub async fn upsert_edge(&mut self, forward_edge: Edge, reverse_edge: Edge) -> (u64, u64) {

        unimplemented!()
    }
}

fn extract_uid(creation_var_name: &str, res: &dgraph_tonic::Response) -> u64 {
    let uid = res.uids.get(creation_var_name);
    match uid {
        Some(uid) => {
            u64::from_str_radix(&uid[2..], 16).expect("uid is not valid hex")
        },
        None => {
            let creation_var_name = format!("q_{}", creation_var_name);
            let v: serde_json::Value = serde_json::from_slice(&res.json).expect("response failed to parse");

            let uid = &v[creation_var_name][0]["uid"].as_str().expect("string");
            u64::from_str_radix(&uid[2..], 16).expect("uid is not valid hex")
        }
    }
}

async fn enforce_transaction<Factory, Txn>(f: Factory) -> dgraph_tonic::Response
    where
        Factory: FnMut() -> Txn + 'static + Unpin,
        Txn: std::future::Future<Output = Result<dgraph_tonic::Response, anyhow::Error>>,
{
    let handle_upsert_err = UpsertErrorHandler {};
    let (response, attempts) = FutureRetry::new(f, handle_upsert_err)
        .await
        .expect("Surfaced an error despite retry strategy while performing an upsert.");

    tracing::info!(message = "Performed upsert", attempts = attempts);

    response
}


pub struct UpsertErrorHandler {}

impl futures_retry::ErrorHandler<anyhow::Error> for UpsertErrorHandler {
    type OutError = anyhow::Error;

    fn handle(&mut self, attempt: usize, e: anyhow::Error) -> RetryPolicy<Self::OutError> {
        let attempt = attempt as u64;
        tracing::warn!(
            message="Failed to enforce transaction",
            error=?e,
            attempt=?attempt,
        );
        match attempt {
            0..=5 => RetryPolicy::Repeat,
            t @ 5..=20 => RetryPolicy::WaitRetry(std::time::Duration::from_millis(10 * t as u64)),
            21..=u64::MAX => RetryPolicy::ForwardError(e),
        }
    }
}
