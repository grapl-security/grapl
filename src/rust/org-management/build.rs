// fn main() -> Result<(), Box<dyn std::error::Error>> {
//
//     tonic_build::compile_protos("proto/orgmanagement.proto")?;
//     Ok(())
// }
fn main() {
    let config = tonic_build::configure();

    config
        .build_server(true)
        .build_client(true)
        .compile(
            &[
                "../../proto/graplinc/grapl/api/orgmanagement/v1/orgmanagement.proto",
            ],
            &["../../proto/"],
        )
        .unwrap_or_else(|e| panic!("orgmanagement protobuf compilation failed: {}", e));
}