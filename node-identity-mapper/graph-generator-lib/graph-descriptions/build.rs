extern crate prost_build;


fn main() {

    let mut config = prost_build::Config::new();

    config.type_attribute(".", "#[derive(Eq)]");
    config.type_attribute(".graph_description.FileDescription", "#[derive(TypedBuilder)]");
    config.type_attribute(".graph_description.ProcessDescription", "#[derive(TypedBuilder)]");
    config.type_attribute(".graph_description.InboundConnection", "#[derive(TypedBuilder)]");
    config.type_attribute(".graph_description.OutboundConnection", "#[derive(TypedBuilder)]");

    config.field_attribute(".graph_description.FileDescription.asset_id", "#[default]");
    config.field_attribute(".graph_description.FileDescription.hostname", "#[default]");
    config.field_attribute(".graph_description.FileDescription.host_ip", "#[default]");

    config.field_attribute(".graph_description.ProcessDescription.asset_id", "#[default]");
    config.field_attribute(".graph_description.ProcessDescription.hostname", "#[default]");
    config.field_attribute(".graph_description.ProcessDescription.host_ip", "#[default]");

    config.field_attribute(".graph_description.InboundConnection.asset_id", "#[default]");
    config.field_attribute(".graph_description.InboundConnection.hostname", "#[default]");
    config.field_attribute(".graph_description.InboundConnection.host_ip", "#[default]");

    config.field_attribute(".graph_description.OutboundConnection.asset_id", "#[default]");
    config.field_attribute(".graph_description.OutboundConnection.hostname", "#[default]");
    config.field_attribute(".graph_description.OutboundConnection.host_ip", "#[default]");


    config
        .compile_protos(&[
            "proto/graph_description.proto",
        ], &["proto/"])
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}