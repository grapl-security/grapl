fn main() {
    let mut config = prost_build::Config::new();

    config.type_attribute(
        ".",
        "#[derive(Eq, serde_derive::Serialize, serde_derive::Deserialize)]",
    );
    config.type_attribute(".graph_description.Edge", "#[derive(Ord, PartialOrd)]");

    config.type_attribute(
        ".graph_description.MergedEdge",
        "#[derive(Ord, PartialOrd)]",
    );

    config
        .compile_protos(&["proto/graph_description.proto"], &["proto/"])
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}
