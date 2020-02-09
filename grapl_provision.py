import json
import pydgraph

from pydgraph import DgraphClient, DgraphClientStub
from grapl_analyzerlib.schemas import *

from grapl_analyzerlib.schemas.schema_builder import ManyToMany


class AnyNodeSchema(NodeSchema):
    @staticmethod
    def self_type() -> str:
        return 'Any'


class RiskSchema(NodeSchema):
    def __init__(self):
        super(RiskSchema, self).__init__()
        (
            self
                .with_str_prop('analyzer_name')
                .with_int_prop('risk_score')
        )

    @staticmethod
    def self_type() -> str:
        return 'Risk'


class LensSchema(NodeSchema):
    def __init__(self):
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

class AssetSchema(NodeSchema):
    def __init__(self):
        super(AssetSchema, self).__init__()
        (
            self.with_str_prop("hostname")
        )

    @staticmethod
    def self_type() -> str:
        return "Asset"


def set_schema(client, schema, engagement=False):
    op = pydgraph.Operation(schema=schema)
    client.alter(op)


def drop_all(client):
    op = pydgraph.Operation(drop_all=True)
    client.alter(op)

def format_schemas(schema_defs):
    schemas = "\n\n".join([schema.to_schema_str() for schema in schema_defs])

    types = "\n\n".join([schema.generate_type() for schema in schema_defs])

    return "\n".join([
        "  # Type Definitions",
        types,
        "\n  # Schema Definitions",
        schemas,
    ])


___local_dg_provision_client = DgraphClient(DgraphClientStub('localhost:9080'))


def provision():


    # drop_all(mclient)
    # drop_all(___local_dg_provision_client)

    schemas = (
        AssetSchema(),
        ProcessSchema(),
        FileSchema(),
        IpConnectionSchema(),
        IpAddressSchema(),
        IpPortSchema(),
        NetworkConnectionSchema(),
        ProcessInboundConnectionSchema(),
        ProcessOutboundConnectionSchema(),
    )

    schema_str = format_schemas(schemas)

    eg_schemas = [s.with_forward_edge('risks', ManyToMany(RiskSchema), 'risky_nodes') for s in schemas]

    risk_schema = RiskSchema()
    lens_schema = LensSchema()
    eg_schemas.extend([risk_schema, lens_schema])
    eg_schema_str = format_schemas(eg_schemas)
    set_schema(___local_dg_provision_client, eg_schema_str)


if __name__ == '__main__':
    drop_all(___local_dg_provision_client)
    provision()