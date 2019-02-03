extern crate prost_build;


fn main() {

    let mut config = prost_build::Config::new();

    config.type_attribute(".", "#[derive(Eq)]");


    config.type_attribute(".graph_description.FileDescription", "#[derive(Builder)]");
    config.type_attribute(".graph_description.ProcessDescription", "#[derive(Builder)]");
    config.type_attribute(".graph_description.InboundConnection", "#[derive(Builder)]");
    config.type_attribute(".graph_description.OutboundConnection", "#[derive(Builder)]");

    config.type_attribute(".graph_description.FileDescription", "#[builder(setter(into))]");
    config.type_attribute(".graph_description.ProcessDescription", "#[builder(setter(into))]");
    config.type_attribute(".graph_description.InboundConnection", "#[builder(setter(into))]");
    config.type_attribute(".graph_description.OutboundConnection", "#[builder(setter(into))]");


    config.field_attribute(".graph_description.FileDescription.node_key", "#[builder(field(private))]");
    config.field_attribute(".graph_description.FileDescription.node_key",
                           "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]");

    config.field_attribute(".graph_description.FileDescription.asset_id", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.hostname", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.host_ip", "#[builder(default)]");


    config.field_attribute(".graph_description.FileDescription.created_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.deleted_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.last_seen_timestamp", "#[builder(default)]");

    config.field_attribute(".graph_description.ProcessDescription.node_key", "#[builder(field(private))]");
    config.field_attribute(".graph_description.ProcessDescription.node_key",
                           "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]");

    config.field_attribute(".graph_description.ProcessDescription.asset_id", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessDescription.hostname", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessDescription.host_ip", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessDescription.image_name", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessDescription.image_path", "#[builder(default)]");

    config.field_attribute(".graph_description.ProcessDescription.created_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessDescription.terminated_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessDescription.last_seen_timestamp", "#[builder(default)]");


    config.field_attribute(".graph_description.InboundConnection.node_key", "#[builder(field(private))]");
    config.field_attribute(".graph_description.InboundConnection.node_key",
                           "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]");

    config.field_attribute(".graph_description.InboundConnection.asset_id", "#[builder(default)]");
    config.field_attribute(".graph_description.InboundConnection.hostname", "#[builder(default)]");
    config.field_attribute(".graph_description.InboundConnection.host_ip", "#[builder(default)]");


    config.field_attribute(".graph_description.InboundConnection.created_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.InboundConnection.terminated_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.InboundConnection.last_seen_timestamp", "#[builder(default)]");

    config.field_attribute(".graph_description.OutboundConnection.node_key", "#[builder(field(private))]");
    config.field_attribute(".graph_description.OutboundConnection.node_key",
                           "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]");

    config.field_attribute(".graph_description.OutboundConnection.asset_id", "#[builder(default)]");
    config.field_attribute(".graph_description.OutboundConnection.hostname", "#[builder(default)]");
    config.field_attribute(".graph_description.OutboundConnection.host_ip", "#[builder(default)]");

    config.field_attribute(".graph_description.OutboundConnection.created_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.OutboundConnection.terminated_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.OutboundConnection.last_seen_timestamp", "#[builder(default)]");

    config
        .compile_protos(&[
            "proto/graph_description.proto",
        ], &["proto/"])
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}