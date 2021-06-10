fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::compile_protos(
        &["../../proto/graplinc/grapl/metrics/metric_types.proto"],
        &["../../proto/"],
    )?;

    Ok(())
}
