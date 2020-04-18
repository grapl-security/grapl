from grapl_analyzerlib.schemas.schema_builder import NodeSchema


class AssetSchema(NodeSchema):
    def __init__(self) -> None:
        super(AssetSchema, self).__init__()
        (
            self.with_str_prop("hostname")
        )

    @staticmethod
    def self_type() -> str:
        return "Asset"
