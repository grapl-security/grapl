from grapl_analyzerlib.schemas.schema_builder import NodeSchema, ManyToOne


class ProcessOutboundConnectionSchema(NodeSchema):
    def __init__(self):
        super(ProcessOutboundConnectionSchema, self).__init__()
        (
            self.with_str_prop("ip_address")
            .with_str_prop("protocol")
            .with_int_prop("created_timestamp")
            .with_int_prop("terminated_timestamp")
            .with_int_prop("last_seen_timestamp")
            .with_int_prop("port")
            .with_forward_edge(
                "connected_over",
                # The source IP + Port that was connected over
                ManyToOne(IpPortSchema),
                "process_connections",
            )
            .with_forward_edge(
                "connected_to",
                # The IP + Port that was connected to
                ManyToOne(IpPortSchema),
                "connections_from",
            )
        )

    @staticmethod
    def self_type() -> str:
        return "ProcessOutboundConnection"


from grapl_analyzerlib.schemas.ip_port_schema import IpPortSchema
