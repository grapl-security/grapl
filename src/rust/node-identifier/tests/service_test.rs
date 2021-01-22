




/// Given:
///     * A Graph with 4 Session-Identifiable Nodes; A, B, C, D
///     * Where A, B share a canonical identity, and C, D share a common identity
///     * Where A, B, have an edge to C, D
/// When:
///     * We identify all nodes
/// Then:
///     * Then, we should have a resulting graph with 2 nodes, and an edge between them

#[tokio::test]
async fn test_service() -> Result<(), Box<dyn std::error::Error>> {
    // todo: I think should_default could in fact be random? It shouldn't matter for this test.
    // let node_identifier = init_local_node_identifier(false).await;

    Ok(())
}
