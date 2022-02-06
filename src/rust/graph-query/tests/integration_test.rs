#![cfg(feature = "integration")]


#[cfg(test)]
mod tests {
    use graph_query::{NodeCell, PropertyFilter, StrEq};
    use dgraph_tonic::Client as DgraphClient;
    use dgraph_tonic::Query;

    #[tokio::test]
    async fn integration_test() -> Result<(), Box<dyn std::error::Error>>{
        let client = DgraphClient::new("http://localhost:9080")?;
        let mut txn = client.new_read_only_txn();

        let query = NodeCell::root()
            .with_property_filters(
                "propname",
                vec![StrEq::eq("foo").boxed(), StrEq::neq("bar").boxed()],
            )
            .with_property_filters("propname", vec![StrEq::eq("baz").boxed()])
            .with_edge_filters(
                "edgename",
                "reverse_edge_name",
                vec![NodeCell::default()
                    .with_property_filters("otherprop", vec![StrEq::neq("baz").boxed()])],
            );
        let (query_string, vars) = query.with_uid(1234);

        // We're only querying DGraph in order to validate the query's syntax
        let _response = txn.query_with_vars(query_string, vars.variable_map()).await?;

        Ok(())
    }
}