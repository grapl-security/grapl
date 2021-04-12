import { Schema } from "../modules/schema_client";

// Grabbed verbatim from the current state of prod on: Apr 12, 2020
export const schemas: Schema[] = [
    {
        type_definition: {
            properties: [
                {
                    name: "uid",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "dgraph.type",
                    primitive: "Str",
                    is_set: true,
                },
                {
                    name: "file_path",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "file_extension",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "file_mime_type",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "file_version",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "file_description",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "file_product",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "file_company",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "file_directory",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "file_hard_links",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "signed",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "signed_status",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "md5_hash",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "sha1_hash",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "sha256_hash",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "file_inode",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "file_size",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "node_key",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "last_index_time",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "risks",
                    primitive: "Risk",
                    is_set: true,
                },
                {
                    name: "bin_file",
                    primitive: "Process",
                    is_set: true,
                },
                {
                    name: "created_files",
                    primitive: "Process",
                    is_set: false,
                },
                {
                    name: "wrote_files",
                    primitive: "Process",
                    is_set: true,
                },
                {
                    name: "read_files",
                    primitive: "Process",
                    is_set: true,
                },
                {
                    name: "deleted_files",
                    primitive: "Process",
                    is_set: false,
                },
            ],
        },
        display_property: "file_path",
        node_type: "File",
    },
    {
        type_definition: {
            properties: [
                {
                    name: "uid",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "dgraph.type",
                    primitive: "Str",
                    is_set: true,
                },
                {
                    name: "created_timestamp",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "terminated_timestamp",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "last_seen_timestamp",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "port",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "ip_address",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "protocol",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "node_key",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "last_index_time",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "risks",
                    primitive: "Risk",
                    is_set: true,
                },
                {
                    name: "connected_over",
                    primitive: "IpPort",
                    is_set: false,
                },
                {
                    name: "connected_to",
                    primitive: "IpPort",
                    is_set: false,
                },
            ],
        },
        display_property: "dgraph_type",
        node_type: "ProcessOutboundConnection",
    },
    {
        type_definition: {
            properties: [
                {
                    name: "uid",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "dgraph.type",
                    primitive: "Str",
                    is_set: true,
                },
                {
                    name: "process_name",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "image_name",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "process_id",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "created_timestamp",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "terminate_time",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "arguments",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "node_key",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "last_index_time",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "risks",
                    primitive: "Risk",
                    is_set: true,
                },
                {
                    name: "children",
                    primitive: "Process",
                    is_set: false,
                },
                {
                    name: "parent",
                    primitive: "Process",
                    is_set: true,
                },
                {
                    name: "created_connections",
                    primitive: "ProcessOutboundConnection",
                    is_set: true,
                },
                {
                    name: "inbound_connections",
                    primitive: "ProcessInboundConnection",
                    is_set: true,
                },
            ],
        },
        display_property: "process_name",
        node_type: "Process",
    },
    {
        type_definition: {
            properties: [
                {
                    name: "uid",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "dgraph.type",
                    primitive: "Str",
                    is_set: true,
                },
                {
                    name: "src_ip_address",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "src_port",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "dst_ip_address",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "dst_port",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "created_timestamp",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "terminated_timestamp",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "last_seen_timestamp",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "node_key",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "last_index_time",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "risks",
                    primitive: "Risk",
                    is_set: true,
                },
                {
                    name: "inbound_ip_connection_to",
                    primitive: "IpAddress",
                    is_set: false,
                },
            ],
        },
        display_property: "dgraph_type",
        node_type: "IpConnection",
    },
    {
        type_definition: {
            properties: [
                {
                    name: "uid",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "dgraph.type",
                    primitive: "Str",
                    is_set: true,
                },
                {
                    name: "port",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "ip_address",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "first_seen_timestamp",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "last_seen_timestamp",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "node_key",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "last_index_time",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "risks",
                    primitive: "Risk",
                    is_set: true,
                },
                {
                    name: "network_connections",
                    primitive: "NetworkConnection",
                    is_set: true,
                },
            ],
        },
        display_property: "dgraph_type",
        node_type: "IpPort",
    },
    {
        type_definition: {
            properties: [
                {
                    name: "uid",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "dgraph.type",
                    primitive: "Str",
                    is_set: true,
                },
                {
                    name: "hostname",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "node_key",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "last_index_time",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "risks",
                    primitive: "Risk",
                    is_set: true,
                },
                {
                    name: "asset_ip",
                    primitive: "IpAddress",
                    is_set: true,
                },
                {
                    name: "asset_processes",
                    primitive: "Process",
                    is_set: false,
                },
                {
                    name: "files_on_asset",
                    primitive: "File",
                    is_set: false,
                },
            ],
        },
        display_property: "hostname",
        node_type: "Asset",
    },
    {
        type_definition: {
            properties: [
                {
                    name: "uid",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "dgraph.type",
                    primitive: "Str",
                    is_set: true,
                },
                {
                    name: "protocol",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "created_timestamp",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "terminated_timestamp",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "port",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "last_seen_timestamp",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "node_key",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "last_index_time",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "risks",
                    primitive: "Risk",
                    is_set: true,
                },
                {
                    name: "bound_port",
                    primitive: "IpPort",
                    is_set: true,
                },
                {
                    name: "bound_ip",
                    primitive: "IpAddress",
                    is_set: true,
                },
            ],
        },
        display_property: "dgraph_type",
        node_type: "ProcessInboundConnection",
    },
    {
        type_definition: {
            properties: [
                {
                    name: "uid",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "dgraph.type",
                    primitive: "Str",
                    is_set: true,
                },
                {
                    name: "src_ip_address",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "src_port",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "dst_ip_address",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "dst_port",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "created_timestamp",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "terminated_timestamp",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "last_seen_timestamp",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "node_key",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "last_index_time",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "risks",
                    primitive: "Risk",
                    is_set: true,
                },
                {
                    name: "inbound_network_connection_to",
                    primitive: "IpPort",
                    is_set: false,
                },
            ],
        },
        display_property: "dgraph_type",
        node_type: "NetworkConnection",
    },
    {
        type_definition: {
            properties: [
                {
                    name: "uid",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "dgraph.type",
                    primitive: "Str",
                    is_set: true,
                },
                {
                    name: "first_seen_timestamp",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "last_seen_timestamp",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "ip_address",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "node_key",
                    primitive: "Str",
                    is_set: false,
                },
                {
                    name: "last_index_time",
                    primitive: "Int",
                    is_set: false,
                },
                {
                    name: "risks",
                    primitive: "Risk",
                    is_set: true,
                },
                {
                    name: "ip_connections",
                    primitive: "IpConnection",
                    is_set: true,
                },
            ],
        },
        display_property: "dgraph_type",
        node_type: "IpAddress",
    },
];
