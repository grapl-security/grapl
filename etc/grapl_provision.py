print(">> Please wait for 'pip done!' before moving to the next cell.")
# !python -m pip install --upgrade -q pip
# !pip install typing-extensions
# !pip install pydgraph
# !pip install grapl_analyzerlib
# !pip install --index-url https://test.pypi.org/simple/ grapl_analyzerlib
print(">> pip done!")

from __future__ import annotations
import json
from typing import *

import pydgraph  # type: ignore
from grapl_analyzerlib.node_types import (
    EdgeRelationship,
    PropPrimitive,
    PropType,
    EdgeT,
)
from grapl_analyzerlib.nodes.base import BaseSchema
from grapl_analyzerlib.prelude import *
from pydgraph import DgraphClient, DgraphClientStub

import boto3

if TYPE_CHECKING:
    from mypy_boto3_dynamodb import DynamoDBServiceResource

print(">> Okay, done setting up imports")
deployment_name = ""  # Fill this in!

assert deployment_name, "Please insert your deployment name here."


def set_schema(client, schema) -> None:
    op = pydgraph.Operation(schema=schema)
    client.alter(op)


def drop_all(client) -> None:
    op = pydgraph.Operation(drop_all=True)
    client.alter(op)


def format_schemas(schema_defs: Sequence["BaseSchema"]) -> str:
    schemas = "\n\n".join([schema.generate_schema() for schema in schema_defs])

    types = "\n\n".join([schema.generate_type() for schema in schema_defs])

    return "\n".join(
        ["  # Type Definitions", types, "\n  # Schema Definitions", schemas]
    )


def query_dgraph_predicate(client: "GraphClient", predicate_name: str) -> Any:
    query = f"""
        schema(pred: {predicate_name}) {{  }}
    """
    txn = client.txn(read_only=True)
    try:
        res = json.loads(txn.query(query).json)["schema"][0]
    finally:
        txn.discard()

    return res


def meta_into_edge(schema, predicate_meta) -> EdgeT:
    if predicate_meta.get("list"):
        return EdgeT(type(schema), BaseSchema, EdgeRelationship.OneToMany)
    else:
        return EdgeT(type(schema), BaseSchema, EdgeRelationship.OneToOne)


def meta_into_property(schema, predicate_meta) -> PropType:
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


def meta_into_predicate(schema, predicate_meta) -> Union[EdgeT, PropType]:
    try:
        if predicate_meta["type"] == "uid":
            return meta_into_edge(schema, predicate_meta)
        else:
            return meta_into_property(schema, predicate_meta)
    except Exception as e:
        raise Exception(f"Failed to convert meta to predicate: {predicate_meta}") from e


def query_dgraph_type(client: "GraphClient", type_name: str) -> List[Any]:
    query = f"""
        schema(type: {type_name}) {{ type }}
    """
    txn = client.txn(read_only=True)
    try:
        res = json.loads(txn.query(query).json)
    finally:
        txn.discard()

    if not res:
        return []
    if not res.get("types"):
        return []

    res = res["types"][0]["fields"]
    predicate_names = []
    for pred in res:
        predicate_names.append(pred["name"])

    predicate_metas = []
    for predicate_name in predicate_names:
        predicate_metas.append(query_dgraph_predicate(client, predicate_name))

    return predicate_metas


def extend_schema(graph_client: GraphClient, schema: "BaseSchema") -> None:
    predicate_metas = query_dgraph_type(graph_client, schema.self_type())

    for predicate_meta in predicate_metas:
        predicate = meta_into_predicate(schema, predicate_meta)
        if isinstance(predicate, PropType):
            schema.add_property(predicate_meta["predicate"], predicate)
        else:
            schema.add_edge(predicate_meta["predicate"], predicate, "")


def store_schema(table, schema: "BaseSchema") -> None:
    for f_edge, (_, r_edge) in schema.get_edges().items():
        if not (f_edge and r_edge):
            print(f"!! We found an edge that is missing a reverse edge: {f_edge}")
            continue

        table.put_item(Item={"f_edge": f_edge, "r_edge": r_edge})
        table.put_item(Item={"f_edge": r_edge, "r_edge": f_edge})
        print(f"stored edge mapping: {f_edge} {r_edge}")


def provision_master_graph(
    master_graph_client: GraphClient, schemas: Sequence["BaseSchema"]
) -> None:
    mg_schema_str = format_schemas(schemas)
    set_schema(master_graph_client, mg_schema_str)


print(">> Okay, done setting up helper functions")
mclient = DgraphClient(DgraphClientStub(deployment_name.lower() + ".dgraph.grapl:9080"))
dynamodb: DynamoDBServiceResource = boto3.resource("dynamodb")
# drop_all(mclient)


def provision_mg(mclient: DgraphClient, dynamodb: DynamoDBServiceResource) -> None:
    # drop_all(mclient)

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
        RiskSchema(),
        LensSchema(),
    )

    for schema in schemas:
        schema.init_reverse()

    for schema in schemas:
        extend_schema(mclient, schema)

    provision_master_graph(mclient, schemas)

    table = dynamodb.Table(f"{deployment_name}-grapl_schema_table")
    for schema in schemas:
        store_schema(table, schema)


provision_mg(mclient=mclient, dynamodb=dynamodb)
print(">> Done provisioning!")

import os
import string
from hashlib import pbkdf2_hmac, sha256
from random import choice, randint
from typing import TYPE_CHECKING


def hash_password(cleartext, salt) -> str:
    hashed = sha256(cleartext).digest()
    return pbkdf2_hmac("sha256", hashed, salt, 512000).hex()


def create_user(username: str, cleartext: str) -> None:
    assert cleartext
    table = dynamodb.Table(deployment_name + "-user_auth_table")

    # We hash before calling 'hashed_password' because the frontend will also perform
    # client side hashing
    cleartext += "f1dafbdcab924862a198deaa5b6bae29aef7f2a442f841da975f1c515529d254"

    cleartext += username

    hashed = sha256(cleartext.encode("utf8")).hexdigest()

    for i in range(0, 5000):
        hashed = sha256(hashed.encode("utf8")).hexdigest()

    salt = os.urandom(16)
    password = hash_password(hashed.encode("utf8"), salt)
    table.put_item(Item={"username": username, "salt": salt, "password": password})


#####
# Fill in your desired username.
#####
username = ""
assert username, "Replace the username with your desired username"
print(f"your username is {username}")

# Your password is automatically generated.
allchar = string.ascii_letters + string.punctuation + string.digits
password = "".join(choice(allchar) for x in range(randint(14, 16)))
print(f"your password is {password}\nstore it somewhere safe!")


create_user(username, password)
password = ""
print(
    """Make sure to clear this cell and restart the notebook to ensure your password does not leak!"""
)
