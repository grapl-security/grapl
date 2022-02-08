#![cfg(feature = "integration")]

#[cfg(test)]
mod tests {
    use dgraph_tonic::{
        Client as DgraphClient,
        Query,
    };
    use graph_query::{
        NodeCell,
        PropertyFilter,
        StrEq,
    };

    #[tokio::test]
    async fn integration_test() -> Result<(), Box<dyn std::error::Error>> {
        let mg_alpha = grapl_config::mg_alphas()
            .pop()
            .expect("Dgraph Alpha not specified.");

        let client = DgraphClient::new(mg_alpha)?;
        let mut txn = client.new_read_only_txn();

        let query = NodeCell::root()
            .with_property_filters(
                "propname",
                vec![StrEq::new("foo", true).boxed(), StrEq::new("bar", false).boxed()],
            )
            .with_property_filters("propname", vec![StrEq::new("baz", true).boxed()])
            .with_edge_filters(
                "edgename",
                "reverse_edge_name",
                vec![NodeCell::default()
                    .with_property_filters("otherprop", vec![StrEq::new("baz", false).boxed()])],
            );
        let (query_string, vars) = query.with_uid(1234);

        // We're only querying DGraph in order to validate the query's syntax
        let _response = txn
            .query_with_vars(query_string, vars.variable_map())
            .await?;

        Ok(())
    }
}
