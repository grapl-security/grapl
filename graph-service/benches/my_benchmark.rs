#[macro_use]
extern crate criterion;
extern crate graph_descriptions;
extern crate dgraph_client;
extern crate graph_merger;

use criterion::Criterion;

use graph_merger::{merge_subgraph, set_process_schema};

use graph_descriptions::{
    EdgeDescription,
    FileDescription,
    GraphDescription,
    IpAddressDescription,
    Node,
    NodeDescription,
    ProcessDescription
};
use graph_descriptions::graph_description::GraphDescriptionProto;
use graph_descriptions::ProcessState;

fn criterion_benchmark(c: &mut Criterion) {

    let mut client =
        dgraph_client::new_client("localhost:9080");


    set_process_schema(&mut client);

    let asset_id = "asset_id".to_owned();
    let asset_id = graph_descriptions::HostIdentifier::AssetId(asset_id);
    let process_state = ProcessState::Created;
    let pid = 100;
    let timestamp = 500;
    let image_name = b"word.exe".to_vec();
    let image_path = b"/home/word.exe".to_vec();

    let child_pid = pid + 20;
    let child_create_time = timestamp + 200;

    let parent_process = ProcessDescription::new (
        asset_id.clone(),
        process_state.clone(),
        pid.clone(),
        timestamp.clone(),
        image_name.clone(),
        image_path.clone(),
    );

    let child_process = ProcessDescription::new (
        asset_id.clone(),
        process_state.clone(),
        child_pid.clone(),
        child_create_time.clone(),
        image_name.clone(),
        image_path.clone(),
    );


    let mut subgraph = GraphDescription::new();
    subgraph.add_edge("children",
                      parent_process.clone_key(),
                      child_process.clone_key());
    subgraph.add_node(parent_process);
    subgraph.add_node(child_process);

    let subgraph = subgraph.into();
    c.bench_function("subgraph_merge", move |b| b.iter(|| {
        merge_subgraph(
            &client,
            &subgraph
        );
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);