from grapl_analyzerlib.schemas.schema_builder import NodeSchema, ManyToMany


class IpPortSchema(NodeSchema):
    def __init__(self):
        super(IpPortSchema, self).__init__()
        (
            self.with_str_prop("ip_address")
            .with_str_prop("protocol")
            .with_int_prop("port")
            .with_int_prop("first_seen_timestamp")
            .with_int_prop("last_seen_timestamp")
            .with_forward_edge(
                "network_connections",
                ManyToMany(NetworkConnectionSchema),
                "connections_from",
            )
        )

    @staticmethod
    def self_type() -> str:
        return "IpPort"


from grapl_analyzerlib.schemas.network_connection_schema import NetworkConnectionSchema
