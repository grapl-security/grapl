from grapl_analyzerlib.schemas.schema_builder import NodeSchema, ManyToMany
from grapl_analyzerlib.schemas.any_node_schema import AnyNodeSchema


class LensSchema(NodeSchema):
    def __init__(self) -> None:
        super(LensSchema, self).__init__()
        (
            self
            .with_str_prop('lens')
            .with_int_prop('score')
            .with_forward_edge('scope', ManyToMany(AnyNodeSchema), 'in_scope')
        )

    @staticmethod
    def self_type() -> str:
        return 'Lens'