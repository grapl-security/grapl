import json
import os
import sys
import time
from typing import List

import boto3
import pydgraph
from grapl_analyzerlib.grapl_client import GraphClient
from grapl_analyzerlib.node_types import (
    EdgeRelationship,
    EdgeT,
    PropPrimitive,
    PropType,
)
from grapl_analyzerlib.nodes.base import BaseSchema
from grapl_analyzerlib.prelude import (
    AssetSchema,
    FileSchema,
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
from grapl_common.env_helpers import DynamoDBResourceFactory
from grapl_common.grapl_logger import get_module_grapl_logger

LOGGER = get_module_grapl_logger(default_log_level="INFO")


def set_schema(client, schema) -> None:
    op = pydgraph.Operation(schema=schema)
    client.alter(op)


def drop_all(client) -> None:
    op = pydgraph.Operation(drop_all=True)
    client.alter(op)


def format_schemas(schema_defs: List["BaseSchema"]) -> str:
    schemas = "\n\n".join([schema.generate_schema() for schema in schema_defs])

    types = "\n\n".join([schema.generate_type() for schema in schema_defs])

    return "\n".join(
        ["  # Type Definitions", types, "\n  # Schema Definitions", schemas]
    )


def query_dgraph_predicate(client: "GraphClient", predicate_name: str):
    query = f"""
        schema(pred: {predicate_name}) {{  }}
    """
    txn = client.txn(read_only=True)
    try:
        res = json.loads(txn.query(query).json)["schema"][0]
    finally:
        txn.discard()

    return res


def meta_into_edge(schema, predicate_meta):
    if predicate_meta.get("list"):
        return EdgeT(type(schema), BaseSchema, EdgeRelationship.OneToMany)
    else:
        return EdgeT(type(schema), BaseSchema, EdgeRelationship.OneToOne)


def meta_into_property(schema, predicate_meta):
    is_set = predicate_meta.get("list")
    type_name = predicate_meta["type"]
    primitive = None
    if type_name == "string":
        primitive = PropPrimitive.Str
    if type_name == "int":
        primitive = PropPrimitive.Int
    if type_name == "bool":
        primitive = PropPrimitive.Bool

    return PropType(primitive, is_set, index=predicate_meta.get("index", []))


def meta_into_predicate(schema, predicate_meta):
    try:
        if predicate_meta["type"] == "uid":
            return meta_into_edge(schema, predicate_meta)
        else:
            return meta_into_property(schema, predicate_meta)
    except Exception as e:
        LOGGER.error(f"Failed to convert meta to predicate: {predicate_meta} {e}")
        raise e


def query_dgraph_type(client: "GraphClient", type_name: str):
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


def extend_schema(graph_client: GraphClient, schema: "BaseSchema"):
    predicate_metas = query_dgraph_type(graph_client, schema.self_type())

    for predicate_meta in predicate_metas:
        predicate = meta_into_predicate(schema, predicate_meta)
        if isinstance(predicate, PropType):
            schema.add_property(predicate_meta["predicate"], predicate)
        else:
            schema.add_edge(predicate_meta["predicate"], predicate, "")


def provision_master_graph(
    master_graph_client: GraphClient, schemas: List["BaseSchema"]
) -> None:
    mg_schema_str = format_schemas(schemas)
    set_schema(master_graph_client, mg_schema_str)


def store_schema(table, schema: "Schema"):
    for f_edge, (_, r_edge) in schema.get_edges().items():
        if not (f_edge and r_edge):
            continue

        table.put_item(Item={"f_edge": f_edge, "r_edge": r_edge})
        table.put_item(Item={"f_edge": r_edge, "r_edge": f_edge})
        LOGGER.info(f"stored edge mapping: {f_edge} {r_edge}")


def provision_mg(mclient) -> None:
    drop_all(mclient)

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
        try:
            extend_schema(mclient, schema)
        except Exception as e:
            LOGGER.warn(f"Failed to extend_schema: {schema} {e}")

    provision_master_graph(mclient, schemas)

    dynamodb = DynamoDBResourceFactory(boto3).from_env()

    table = dynamodb.Table("local-grapl-grapl_schema_table")
    for schema in schemas:
        try:
            store_schema(table, schema)
        except Exception as e:
            LOGGER.warn(f"storing schema: {schema} {table} {e}")


DEPLOYMENT_NAME = "local-grapl"


def validate_environment():
    """Ensures that the required environment variables are present in the environment.

    Other code actually reads the variables later.
    """
    required = [
        "AWS_REGION",
        "DYNAMODB_ACCESS_KEY_ID",
        "DYNAMODB_ACCESS_KEY_SECRET",
        "DYNAMODB_ENDPOINT",
    ]

    missing = [var for var in required if var not in os.environ]

    if missing:
        print(
            f"The following environment variables are required, but are not present: {missing}"
        )
        sys.exit(1)


if __name__ == "__main__":
    validate_environment()

    time.sleep(5)
    graph_client = GraphClient()

    LOGGER.debug("Provisioning graph database")

    for i in range(0, 150):
        try:
            drop_all(graph_client)
            break
        except Exception as e:
            time.sleep(2)
            if i > 20:
                LOGGER.error("Failed to drop", e)

    mg_succ = False

    LOGGER.info("Starting to provision master graph")
    for i in range(0, 150):
        try:
            if not mg_succ:
                time.sleep(1)
                provision_mg(
                    graph_client,
                )
                mg_succ = True
                LOGGER.info("Provisioned master graph")
                break
        except Exception as e:
            if i > 10:
                LOGGER.error(f"mg provision failed with: {e}")

    LOGGER.info("Completed provisioning")
