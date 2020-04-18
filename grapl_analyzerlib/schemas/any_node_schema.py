from grapl_analyzerlib.schemas.schema_builder import NodeSchema


class AnyNodeSchema(NodeSchema):
    @staticmethod
    def self_type() -> str:
        return 'Any'