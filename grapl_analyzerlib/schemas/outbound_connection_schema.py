from grapl_analyzerlib.schemas.schema_builder import NodeSchema


class OutboundConnectionSchema(NodeSchema):
    def __init__(self):
        super(OutboundConnectionSchema, self).__init__()
        (
            self
            .with_int_prop('create_time')
            .with_int_prop('terminate_time')
            .with_int_prop('last_seen_time')
            .with_str_prop('ip')
            .with_str_prop('port')
            .with_forward_edge('external_connection', ExternalIpSchema)
        )

    @staticmethod
    def self_type() -> str:
        return "OutboundConnection"


from grapl_analyzerlib.schemas import ExternalIpSchema
