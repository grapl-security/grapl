from __future__ import annotations

import json
import logging
import os
import sys
from hashlib import pbkdf2_hmac, sha256
from typing import TYPE_CHECKING, Any, List, Sequence, Union

import boto3
import pydgraph
from grapl_analyzerlib.node_types import (
    EdgeRelationship,
    EdgeT,
    PropPrimitive,
    PropType,
)
from grapl_analyzerlib.prelude import (
    AssetSchema,
    BaseSchema,
    FileSchema,
    GraphClient,
    IpAddressSchema,
    IpConnectionSchema,
    IpPortSchema,
    LensSchema,
    NetworkConnectionSchema,
    ProcessInboundConnectionSchema,
    ProcessOutboundConnectionSchema,
    ProcessSchema,
    RiskSchema,
)
from grapl_analyzerlib.schema import Schema
from grapl_common.provision import store_schema_properties  # type: ignore
from typing_extensions import TypedDict

if TYPE_CHECKING:
    from mypy_boto3_dynamodb import DynamoDBServiceResource
    from mypy_boto3_secretsmanager import Client as SecretsmanagerClient

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(os.getenv("GRAPL_LOG_LEVEL", "INFO"))
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

DEPLOYMENT_NAME = os.environ["DEPLOYMENT_NAME"]
GRAPL_TEST_USER_NAME = os.environ["GRAPL_TEST_USER_NAME"]


def _set_schema(graph_client: GraphClient, schema: str) -> None:
    op = pydgraph.Operation(schema=schema)
    LOGGER.info(f"setting dgraph schema {schema}")
    graph_client.alter(op)


def _format_schemas(schema_defs: Sequence[BaseSchema]) -> str:
    schemas = "\n\n".join([schema.generate_schema() for schema in schema_defs])

    types = "\n\n".join([schema.generate_type() for schema in schema_defs])

    return "\n".join(
        ["  # Type Definitions", types, "\n  # Schema Definitions", schemas]
    )


def _query_dgraph_predicate(client: "GraphClient", predicate_name: str) -> Any:
    query = f"""
        schema(pred: {predicate_name}) {{  }}
    """
    txn = client.txn(read_only=True)
    try:
        res = json.loads(txn.query(query).json)["schema"][0]
    finally:
        txn.discard()

    return res


def _meta_into_edge(schema, predicate_meta) -> EdgeT:
    if predicate_meta.get("list"):
        return EdgeT(type(schema), BaseSchema, EdgeRelationship.OneToMany)
    else:
        return EdgeT(type(schema), BaseSchema, EdgeRelationship.OneToOne)


def _meta_into_property(predicate_meta) -> PropType:
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


def _meta_into_predicate(schema, predicate_meta) -> Union[EdgeT, PropType]:
    try:
        if predicate_meta["type"] == "uid":
            return _meta_into_edge(schema, predicate_meta)
        else:
            return _meta_into_property(predicate_meta)
    except Exception as e:
        raise Exception(f"Failed to convert meta to predicate: {predicate_meta}") from e


def _query_dgraph_type(graph_client: GraphClient, type_name: str) -> List[Any]:
    query = f"""
        schema(type: {type_name}) {{ type }}
    """
    txn = graph_client.txn(read_only=True)
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
        predicate_metas.append(_query_dgraph_predicate(graph_client, predicate_name))

    return predicate_metas


def _extend_schema(graph_client: GraphClient, schema: BaseSchema) -> None:
    predicate_metas = _query_dgraph_type(graph_client, schema.self_type())

    for predicate_meta in predicate_metas:
        predicate = _meta_into_predicate(schema, predicate_meta)
        if isinstance(predicate, PropType):
            schema.add_property(predicate_meta["predicate"], predicate)
        else:
            schema.add_edge(predicate_meta["predicate"], predicate, "")


def _store_schema(dynamodb, schema: "Schema"):
    grapl_schema_table = dynamodb.Table(
        os.environ["DEPLOYMENT_NAME"] + "-grapl_schema_table"
    )

    for f_edge, (_, r_edge) in schema.get_edges().items():
        if not (f_edge and r_edge):
            LOGGER.warn(f"!! We found an edge that is missing a reverse edge: {f_edge}")
            continue

    grapl_schema_table.put_item(Item={"f_edge": f_edge, "r_edge": r_edge})
    grapl_schema_table.put_item(Item={"f_edge": r_edge, "r_edge": f_edge})
    LOGGER.info(f"stored edge mapping: {f_edge} {r_edge}")


def _provision_graph(
    graph_client: GraphClient, dynamodb: DynamoDBServiceResource
) -> None:
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
        _extend_schema(graph_client, schema)

    schema_str = _format_schemas(schemas)
    _set_schema(graph_client, schema_str)

    table: DynamoDBServiceResource.Table = dynamodb.Table(
        f"{DEPLOYMENT_NAME}-grapl_schema_table"
    )
    props_table: DynamoDBServiceResource.Table = dynamodb.Table(
        f"{DEPLOYMENT_NAME}-grapl_schema_properties_table"
    )
    for schema in schemas:
        _store_schema(dynamodb, schema)
        store_schema_properties(props_table, schema)


def _hash_password(cleartext, salt) -> str:
    hashed = sha256(cleartext).digest()
    return pbkdf2_hmac("sha256", hashed, salt, 512000).hex()


def _create_user(
    dynamodb: DynamoDBServiceResource, username: str, cleartext: str
) -> None:
    assert cleartext
    table = dynamodb.Table(DEPLOYMENT_NAME + "-user_auth_table")

    # We hash before calling 'hashed_password' because the frontend will also perform
    # client side hashing
    cleartext += "f1dafbdcab924862a198deaa5b6bae29aef7f2a442f841da975f1c515529d254"

    cleartext += username

    hashed = sha256(cleartext.encode("utf8")).hexdigest()

    for _ in range(0, 5000):
        hashed = sha256(hashed.encode("utf8")).hexdigest()

    salt = os.urandom(16)
    password = _hash_password(hashed.encode("utf8"), salt)
    table.put_item(Item={"username": username, "salt": salt, "password": password})


def _retrieve_test_user_password(
    secretsmanager: SecretsmanagerClient, deployment_name: str
):
    return secretsmanager.get_secret_value(
        SecretId=f"{deployment_name}-TestUserPassword"
    )["SecretString"]


def provision(event: Any = None, context: Any = None):
    graph_client = GraphClient()
    dynamodb: DynamoDBServiceResource = boto3.resource("dynamodb")
    secretsmanager: SecretsmanagerClient = boto3.client("secretsmanager")

    _provision_graph(graph_client=graph_client, dynamodb=dynamodb)

    password = _retrieve_test_user_password(secretsmanager, DEPLOYMENT_NAME)
    _create_user(dynamodb=dynamodb, username=GRAPL_TEST_USER_NAME, cleartext=password)


if __name__ == "__main__":
    provision()
