from grapl_analyzerlib.schemas.schema_builder import NodeSchema, ManyToOne


class IpConnectionSchema(NodeSchema):
    def __init__(self):
        super(IpConnectionSchema, self).__init__()
        (
            self.with_str_prop("src_ip_address")
            .with_str_prop("src_port")
            .with_str_prop("dst_ip_address")
            .with_str_prop("dst_port")
            .with_int_prop("created_timestamp")
            .with_int_prop("terminated_timestamp")
            .with_int_prop("last_seen_timestamp")
            .with_forward_edge(
                "inbound_connection_to",
                ManyToOne(IpAddressSchema),
                "network_connections_from",
            )
        )

    @staticmethod
    def self_type() -> str:
        return "IpConnection"


from grapl_analyzerlib.schemas.ip_address_schema import IpAddressSchema
