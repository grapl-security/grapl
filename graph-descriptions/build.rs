extern crate prost_build;


fn main() {

    let mut config = prost_build::Config::new();

    config.type_attribute(".", "#[derive(Eq, Serialize, Deserialize)]");


    config.type_attribute(".graph_description.Asset", "#[derive(Builder)]");
    config.type_attribute(".graph_description.File", "#[derive(Builder)]");
    config.type_attribute(".graph_description.Process", "#[derive(Builder)]");
    config.type_attribute(".graph_description.ProcessInboundConnection", "#[derive(Builder)]");
    config.type_attribute(".graph_description.ProcessOutboundConnection", "#[derive(Builder)]");
    config.type_attribute(".graph_description.IpAddress", "#[derive(Builder)]");
    config.type_attribute(".graph_description.IpPort", "#[derive(Builder)]");
    config.type_attribute(".graph_description.NetworkConnection", "#[derive(Builder)]");
    config.type_attribute(".graph_description.IpConnection", "#[derive(Builder)]");

    config.type_attribute(".graph_description.Asset", "#[builder(setter(into))]");
    config.type_attribute(".graph_description.File", "#[builder(setter(into))]");
    config.type_attribute(".graph_description.Process", "#[builder(setter(into))]");
    config.type_attribute(".graph_description.ProcessInboundConnection", "#[builder(setter(into))]");
    config.type_attribute(".graph_description.ProcessOutboundConnection", "#[builder(setter(into))]");
    config.type_attribute(".graph_description.IpAddress", "#[builder(setter(into))]");
    config.type_attribute(".graph_description.IpPort", "#[builder(setter(into))]");
    config.type_attribute(".graph_description.NetworkConnection", "#[builder(setter(into))]");
    config.type_attribute(".graph_description.IpConnection", "#[builder(setter(into))]");


    config.field_attribute(".graph_description.File.node_key", "#[builder(field(private))]");
    config.field_attribute(".graph_description.File.node_key",
                           "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]");

    config.field_attribute(".graph_description.File.created_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.File.deleted_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.File.last_seen_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.File.file_name", "#[builder(default)]");
    config.field_attribute(".graph_description.File.file_path", "#[builder(default)]");
    config.field_attribute(".graph_description.File.file_extension", "#[builder(default)]");
    config.field_attribute(".graph_description.File.file_mime_type", "#[builder(default)]");
    config.field_attribute(".graph_description.File.file_size", "#[builder(default)]");
    config.field_attribute(".graph_description.File.file_version", "#[builder(default)]");
    config.field_attribute(".graph_description.File.file_description", "#[builder(default)]");
    config.field_attribute(".graph_description.File.file_product", "#[builder(default)]");
    config.field_attribute(".graph_description.File.file_company", "#[builder(default)]");
    config.field_attribute(".graph_description.File.file_directory", "#[builder(default)]");
    config.field_attribute(".graph_description.File.file_inode", "#[builder(default)]");
    config.field_attribute(".graph_description.File.file_hard_links", "#[builder(default)]");
    config.field_attribute(".graph_description.File.md5_hash", "#[builder(default)]");
    config.field_attribute(".graph_description.File.sha1_hash", "#[builder(default)]");
    config.field_attribute(".graph_description.File.sha256_hash", "#[builder(default)]");
    config.field_attribute(".graph_description.File.asset_id", "#[builder(default)]");
    config.field_attribute(".graph_description.File.hostname", "#[builder(default)]");
    config.field_attribute(".graph_description.File.host_ip", "#[builder(default)]");

    config.field_attribute(".graph_description.Process.node_key", "#[builder(field(private))]");
    config.field_attribute(".graph_description.Process.node_key",
                           "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]");

    config.field_attribute(".graph_description.Process.process_id", "#[builder(default)]");
    config.field_attribute(".graph_description.Process.process_guid", "#[builder(default)]");
    config.field_attribute(".graph_description.Process.created_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.Process.terminated_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.Process.last_seen_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.Process.process_name", "#[builder(default)]");
    config.field_attribute(".graph_description.Process.process_command_line", "#[builder(default)]");
    config.field_attribute(".graph_description.Process.process_integrity_level", "#[builder(default)]");
    config.field_attribute(".graph_description.Process.operating_system", "#[builder(default)]");

    config.field_attribute(".graph_description.Process.asset_id", "#[builder(default)]");
    config.field_attribute(".graph_description.Process.hostname", "#[builder(default)]");
    config.field_attribute(".graph_description.Process.host_ip", "#[builder(default)]");


    config.field_attribute(".graph_description.ProcessInboundConnection.node_key", "#[builder(field(private))]");
    config.field_attribute(".graph_description.ProcessInboundConnection.node_key",
                           "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]");

    config.field_attribute(".graph_description.ProcessInboundConnection.asset_id", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessInboundConnection.hostname", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessInboundConnection.host_ip", "#[builder(default)]");


    config.field_attribute(".graph_description.ProcessInboundConnection.created_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessInboundConnection.terminated_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessInboundConnection.last_seen_timestamp", "#[builder(default)]");

    config.field_attribute(".graph_description.ProcessOutboundConnection.node_key", "#[builder(field(private))]");
    config.field_attribute(".graph_description.ProcessOutboundConnection.node_key",
                           "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]");

    config.field_attribute(".graph_description.ProcessOutboundConnection.asset_id", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessOutboundConnection.hostname", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessOutboundConnection.host_ip", "#[builder(default)]");

    config.field_attribute(".graph_description.ProcessOutboundConnection.created_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessOutboundConnection.terminated_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.ProcessOutboundConnection.last_seen_timestamp", "#[builder(default)]");

    config.field_attribute(".graph_description.Asset.node_key", "#[builder(field(private))]");
    config.field_attribute(".graph_description.Asset.node_key",
                           "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]");

    config.field_attribute(".graph_description.Asset.asset_id", "#[builder(default)]");
    config.field_attribute(".graph_description.Asset.ip_address", "#[builder(default)]");
    config.field_attribute(".graph_description.Asset.hostname", "#[builder(default)]");
    config.field_attribute(".graph_description.Asset.mac_address", "#[builder(default)]");
    config.field_attribute(".graph_description.Asset.first_seen_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.Asset.last_seen_timestamp", "#[builder(default)]");


    config.field_attribute(".graph_description.IpAddress.node_key", "#[builder(field(private))]");
    config.field_attribute(".graph_description.IpAddress.node_key",
                           "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]");
    config.field_attribute(".graph_description.IpAddress.first_seen_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.IpAddress.last_seen_timestamp", "#[builder(default)]");


    config.field_attribute(".graph_description.IpPort.node_key", "#[builder(field(private))]");
    config.field_attribute(".graph_description.IpPort.node_key",
                           "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]");

    config.field_attribute(".graph_description.IpPort.first_seen_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.IpPort.last_seen_timestamp", "#[builder(default)]");


    config.field_attribute(".graph_description.NetworkConnection.node_key", "#[builder(field(private))]");
    config.field_attribute(".graph_description.NetworkConnection.node_key",
                           "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]");

    config.field_attribute(".graph_description.NetworkConnection.created_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.NetworkConnection.terminated_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.NetworkConnection.last_seen_timestamp", "#[builder(default)]");

    config.field_attribute(".graph_description.IpConnection.node_key", "#[builder(field(private))]");
    config.field_attribute(".graph_description.IpConnection.node_key",
                           "#[builder(default = \"::uuid::Uuid::new_v4().to_string()\")]");

    config.field_attribute(".graph_description.IpConnection.created_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.IpConnection.terminated_timestamp", "#[builder(default)]");
    config.field_attribute(".graph_description.IpConnection.last_seen_timestamp", "#[builder(default)]");


    config
        .compile_protos(&[
            "proto/graph_description.proto",
        ], &["proto/"])
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}
