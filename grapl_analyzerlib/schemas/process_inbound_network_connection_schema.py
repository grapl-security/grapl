from grapl_analyzerlib.schemas.schema_builder import NodeSchema, ManyToMany


class ProcessInboundConnectionSchema(NodeSchema):
    def __init__(self):
        super(ProcessInboundConnectionSchema, self).__init__()
        (
            self.with_str_prop("ip_address")
            .with_str_prop("protocol")
            .with_int_prop("created_timestamp")
            .with_int_prop("terminated_timestamp")
            .with_int_prop("last_seen_timestamp")
            .with_int_prop("port")
            .with_forward_edge(
                "bound_port",
                # The IP + Port that was bound
                ManyToMany(IpPortSchema),
                "bound_by",
            )
            .with_forward_edge(
                "bound_ip",
                # The IP that was bound
                ManyToMany(IpAddressSchema),
                "bound_ports",
            )
        )

    @staticmethod
    def self_type() -> str:
        return "ProcessInboundConnection"


from grapl_analyzerlib.schemas.ip_address_schema import IpAddressSchema
from grapl_analyzerlib.schemas.ip_port_schema import IpPortSchema
