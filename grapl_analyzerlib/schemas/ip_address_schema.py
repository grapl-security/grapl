from grapl_analyzerlib.schemas.schema_builder import NodeSchema, ManyToMany


class IpAddressSchema(NodeSchema):
    def __init__(self):
        super(IpAddressSchema, self).__init__()

        (
            self.with_str_prop("ip_address")
            .with_int_prop("first_seen_timestamp")
            .with_int_prop("last_seen_timestamp")
            .with_forward_edge(
                "ip_connections", ManyToMany(IpConnectionSchema), "connecting_ips"
            )
        )

    @staticmethod
    def self_type() -> str:
        return "IpAddress"


from grapl_analyzerlib.schemas.ip_connection_schema import IpConnectionSchema
