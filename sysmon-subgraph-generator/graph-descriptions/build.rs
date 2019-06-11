extern crate prost_build;


fn main() {

    let mut config = prost_build::Config::new();

    config.type_attribute(".", "#[derive(Eq, Serialize, Deserialize)]");


    config.type_attribute(".graph_description.AssetDescription", "#[derive(Builder)]");
    config.type_attribute(".graph_description.FileDescription", "#[derive(Builder)]");
    config.type_attribute(".graph_description.ProcessDescription", "#[derive(Builder)]");
    config.type_attribute(".graph_description.InboundConnection", "#[derive(Builder)]");
    config.type_attribute(".graph_description.OutboundConnection", "#[derive(Builder)]");

    config.type_attribute(".graph_description.AssetDescription", "#[builder(setter(into))]");
    config.type_attribute(".graph_description.FileDescription", "#[builder(setter(into))]");
    config.type_attribute(".graph_description.ProcessDescription", "#[builder(setter(into))]");
    config.type_attribute(".graph_description.InboundConnection", "#[builder(setter(into))]");
    config.type_attribute(".graph_description.OutboundConnection", "#[builder(setter(into))]");


    config.field_attribute(".graph_description.FileDescription.node_key", "#[builder(field(private))]");
    config.field_attribute(".graph_description.FileDescription.node_key",
                           "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]");

    config.field_attribute(".graph_description.FileDescription.created_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.deleted_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.last_seen_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.file_name", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.file_path", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.file_extension", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.file_mime_type", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.file_size", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.file_version", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.file_description", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.file_product", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.file_company", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.file_directory", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.file_inode", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.file_hard_links", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.md5_hash", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.sha1_hash", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.sha256_hash", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.asset_id", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.hostname", "#[builder(default)]");
    config.field_attribute(".graph_description.FileDescription.host_ip", "#[builder(default)]");

    config.field_attribute(".graph_description.ProcessDescription.node_key", "#[builder(field(private))]");
    config.field_attribute(".graph_description.ProcessDescription.node_key",
                           "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]");

    config.field_attribute(".graph_description.ProcessDescription.process_id", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessDescription.process_guid", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessDescription.created_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessDescription.terminated_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessDescription.last_seen_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessDescription.process_name", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessDescription.process_command_line", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessDescription.process_integrity_level", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessDescription.operating_system", "#[builder(default)]");


    config.field_attribute(".graph_description.ProcessDescription.asset_id", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessDescription.hostname", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessDescription.host_ip", "#[builder(default)]");


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

    config.field_attribute(".graph_description.AssetDescription.node_key", "#[builder(field(private))]");
    config.field_attribute(".graph_description.AssetDescription.node_key",
                           "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]");

    config.field_attribute(".graph_description.AssetDescription.asset_id", "#[builder(default)]");
    config.field_attribute(".graph_description.AssetDescription.host_name", "#[builder(default)]");
    config.field_attribute(".graph_description.AssetDescription.host_domain", "#[builder(default)]");
    config.field_attribute(".graph_description.AssetDescription.host_fqdn", "#[builder(default)]");
    config.field_attribute(".graph_description.AssetDescription.host_local_mac", "#[builder(default)]");
    config.field_attribute(".graph_description.AssetDescription.host_ip", "#[builder(default)]");
    config.field_attribute(".graph_description.AssetDescription.operating_system", "#[builder(default)]");


    config
        .compile_protos(&[
            "proto/graph_description.proto",
        ], &["proto/"])
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}
