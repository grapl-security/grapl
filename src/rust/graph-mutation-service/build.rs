fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tonic_build::configure()
    //     .type_attribute(".", "#[derive(Eq)]")
    //     .build_server(true)
    //     // .out_dir("src/")
    //     .compile(
    //         &["../../proto/graplinc/grapl/api/graph/v1beta1/graph_mutation.proto"],
    //         &["../../proto/"],
    //     )?;

    Ok(())
    // config
    //     .compile_protos(
    //         &["../../proto/graplinc/grapl/api/graph/v1beta1/graph_mutation.proto"],
    //         &["../../proto/"],
    //     )
    //     .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}
