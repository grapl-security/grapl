"""
A note: this should eventually be moved out to a lib on top of `grapl_analyzerlib`.
grapl-common is beneath grapl_analyzerlib in the stack, so that's a bad candidate.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, Dict, List, cast
from grapl_analyzerlib.node_types import PropType
from grapl_analyzerlib.provision.meta_into import meta_into_predicate
from grapl_analyzerlib.provision.queries import query_dgraph_type

from grapl_common.resources import known_dynamodb_tables
from grapl_common.grapl_logger import get_module_grapl_logger
from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.schema import Schema
from grapl_analyzerlib.nodes.base import BaseSchema
import pydgraph

if TYPE_CHECKING:
    from mypy_boto3_dynamodb.service_resource import Table


LOGGER = get_module_grapl_logger()


"""
# TODO: Resurrect these once PyType supports TypedDict, or once analyzerlib is mypy'd
from typing_extensions import TypedDict
class SchemaPropertyDict(TypedDict):
    name: str
    primitive: str
    is_set: bool
    # Future TODO: Perhaps also specify edge data here,
    # like `is_edge: <No | Fwd | Reverse>` or something


class SchemaDict(TypedDict):
    properties: List[SchemaPropertyDict]
"""
SchemaPropertyDict = Dict[str, Any]
SchemaDict = Dict[str, Any]


# just some aliases
get_schema_table = known_dynamodb_tables.schema_table
get_schema_properties_table = known_dynamodb_tables.schema_properties_table


def store_schema_properties(table: Table, schema: Schema) -> None:
    properties: List[SchemaPropertyDict] = [
        {
            "name": prop_name,
            # Special case: treat uids as int
            "primitive": prop_type.primitive.name if prop_name != "uid" else "Int",
            "is_set": prop_type.is_set,
        }
        for prop_name, prop_type in schema.get_properties().items()
    ]

    # Don't send over these edges
    denylist_edges = ("in_scope",)
    edges: List[SchemaPropertyDict] = [
        {
            "name": edge_name,
            "primitive": edge_tuple[
                0
            ].dest.self_type(),  # Forward edge goes to this type
            "is_set": edge_tuple[0].is_to_many(),
        }
        for edge_name, edge_tuple in schema.forward_edges.items()
        if edge_name not in denylist_edges
    ]
    type_definition: SchemaDict = {"properties": properties + edges}
    table.put_item(
        Item={
            "node_type": schema.self_type(),
            # Dynamodb doesn't like my fancy typedict
            "type_definition": cast(Dict[str, Any], type_definition),
            "display_property": schema.get_display_property(),
        }
    )


def store_schema(table: Table, schema: Schema) -> None:
    for f_edge, (edge_t, r_edge) in schema.get_edges().items():
        if not (f_edge and r_edge):
            LOGGER.warn(f"missing {f_edge} {r_edge} for {schema.self_type()}")
            continue
        table.put_item(
            Item={
                "f_edge": f_edge,
                "r_edge": r_edge,
                "relationship": int(edge_t.rel),
            }
        )

        table.put_item(
            Item={
                "f_edge": r_edge,
                "r_edge": f_edge,
                "relationship": int(edge_t.rel.reverse()),
            }
        )


def format_schemas(schema_defs: List[BaseSchema]) -> str:
    schemas = "\n\n".join([schema.generate_schema() for schema in schema_defs])

    types = "\n\n".join([schema.generate_type() for schema in schema_defs])

    return "\n".join(
        ["  # Type Definitions", types, "\n  # Schema Definitions", schemas]
    )


def set_schema(client: GraphClient, schema: str) -> None:
    op = pydgraph.Operation(schema=schema, run_in_background=True)
    LOGGER.info(f"setting dgraph schema {schema}")
    client.alter(op)


def _get_reverse_edge(schema_table: Table, schema: BaseSchema, f_edge) -> str:
    edge_res = schema_table.get_item(Key={"f_edge": f_edge})["Item"]
    return edge_res["r_edge"]


def extend_schema(
    schema_table: Table, graph_client: GraphClient, schema: BaseSchema
) -> None:
    predicate_metas = query_dgraph_type(graph_client, schema.self_type())
    for predicate_meta in predicate_metas:
        predicate = meta_into_predicate(schema_table, schema, predicate_meta)
        if isinstance(predicate, PropType):
            schema.add_property(predicate_meta["predicate"], predicate)
        else:
            r_edge = _get_reverse_edge(
                schema_table, schema, predicate_meta["predicate"]
            )
            schema.add_edge(predicate_meta["predicate"], predicate, r_edge)
