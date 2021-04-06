from __future__ import annotations

from typing import TYPE_CHECKING, Any, Dict, List, cast

from typing_extensions import TypedDict

if TYPE_CHECKING:
    from mypy_boto3_dynamodb.service_resource import Table


class SchemaPropertyDict(TypedDict):
    name: str
    primitive: str
    is_set: bool


# Schema is defined in grapl_analyzerlib, which is actually below `-common` in the stack.
# I've put this code in `-common` so that it can be shared by:
# - Model Plugin Deployer
# - grapl_provision
# - provisioner lambda
# ( and grapl-analyzerlib seems like the wrong place to do that )
Schema = Any


class SchemaDict(TypedDict):
    properties: List[SchemaPropertyDict]


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
    type_definition: SchemaDict = {"properties": properties}
    table.put_item(
        Item={
            "node_type": schema.self_type(),
            # Dynamodb doesn't like my fancy typedict
            "type_definition": cast(Dict[str, Any], type_definition),
        }
    )
