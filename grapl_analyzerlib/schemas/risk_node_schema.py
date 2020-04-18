from grapl_analyzerlib.schemas.schema_builder import NodeSchema


class RiskSchema(NodeSchema):
    def __init__(self) -> None:
        super(RiskSchema, self).__init__()
        (
            self
            .with_str_prop("analyzer_name")
            .with_int_prop("risk_score")
        )

    @staticmethod
    def self_type() -> str:
        return "Risk"


if __name__ == '__main__':
    from grapl_analyzerlib.schemas.schema_builder import generate_plugin_query, generate_plugin_view
    schema = RiskSchema()
    query = generate_plugin_query(schema)
    view = generate_plugin_view(schema)

    print(query)
    print(view)
