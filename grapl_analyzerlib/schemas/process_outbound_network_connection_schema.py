from grapl_analyzerlib.schemas.schema_builder import NodeSchema, ManyToOne


class ProcessOutboundNetworkConnectionSchema(NodeSchema):
    def __init__(self):
        super(ProcessOutboundNetworkConnectionSchema, self).__init__()
        (
            self.with_str_prop("ip_address")
            .with_str_prop("protocol")
            .with_int_prop("created_timestamp")
            .with_int_prop("terminated_timestamp")
            .with_int_prop("last_seen_timestamp")
            .with_int_prop("port")
            .with_forward_edge(
                "connected_over",
                # The IP + Port that was connected to
                ManyToOne(IpPortSchema),
                "process_connections",
            )
        )

    @staticmethod
    def self_type() -> str:
        return "ProcessOutboundNetworkConnection"


from grapl_analyzerlib.schemas.ip_port_schema import IpPortSchema
