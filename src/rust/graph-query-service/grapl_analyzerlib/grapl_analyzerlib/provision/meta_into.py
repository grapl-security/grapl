from __future__ import annotations

from typing import TYPE_CHECKING
from grapl_analyzerlib.node_types import (
    EdgeRelationship,
    EdgeT,
    PropPrimitive,
    PropType,
)

from grapl_analyzerlib.schema import Schema

if TYPE_CHECKING:
    from mypy_boto3_dynamodb.service_resource import Table


from typing import Union


def meta_into_edge(schema_table: Table, schema: Schema, f_edge) -> EdgeT:
    edge_res = schema_table.get_item(Key={"f_edge": f_edge})["Item"]
    edge_t = schema.edges[f_edge][0]  # type: EdgeT

    return EdgeT(type(schema), edge_t.dest, EdgeRelationship(edge_res["relationship"]))


def meta_into_property(predicate_meta) -> PropType:
    is_set = predicate_meta.get("list")
    type_name = predicate_meta["type"]
    primitive = None
    if type_name == "string":
        primitive = PropPrimitive.Str
    if type_name == "int":
        primitive = PropPrimitive.Int
    if type_name == "bool":
        primitive = PropPrimitive.Bool

    assert primitive is not None
    return PropType(primitive, is_set, index=predicate_meta.get("index", []))


def meta_into_predicate(
    schema_table: Table, schema, predicate_meta
) -> Union[EdgeT, PropType]:
    try:
        if predicate_meta["type"] == "uid":
            return meta_into_edge(schema_table, schema, predicate_meta["predicate"])
        else:
            return meta_into_property(predicate_meta)
    except Exception as e:
        raise Exception(f"Failed to convert meta to predicate: {predicate_meta}") from e
