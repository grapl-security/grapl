from grapl_analyzerlib.schemas.schema_builder import NodeSchema


class ExternalIpSchema(NodeSchema):
    def __init__(self):
        super(ExternalIpSchema, self).__init__()
        (
            self
            .with_str_prop('external_ip')
        )

    @staticmethod
    def self_type() -> str:
        return "ExternalIp"


