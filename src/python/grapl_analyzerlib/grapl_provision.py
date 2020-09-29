import json
import os
import logging
import sys
import threading
import time
from hashlib import sha256, pbkdf2_hmac
from typing import List, Any
from uuid import uuid4

import boto3
import botocore
import pydgraph
from pydgraph import DgraphClient, DgraphClientStub

from grapl_analyzerlib.grapl_client import MasterGraphClient, GraphClient
from grapl_analyzerlib.node_types import (
    EdgeRelationship,
    PropPrimitive,
    PropType,
    EdgeT,
)
from grapl_analyzerlib.nodes.base import BaseSchema
from grapl_analyzerlib.prelude import (
    AssetSchema,
    ProcessSchema,
    FileSchema,
    IpConnectionSchema,
    IpAddressSchema,
    IpPortSchema,
    NetworkConnectionSchema,
    ProcessInboundConnectionSchema,
    ProcessOutboundConnectionSchema,
    LensSchema,
    RiskSchema,
)
from grapl_analyzerlib.schema import Schema

GRAPL_LOG_LEVEL = os.getenv("GRAPL_LOG_LEVEL")
LEVEL = "ERROR" if GRAPL_LOG_LEVEL is None else GRAPL_LOG_LEVEL
LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(LEVEL)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))


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


def meta_into_edge(dynamodb, schema, f_edge):
    table = dynamodb.Table(os.environ["BUCKET_PREFIX"] + "-grapl_schema_table")
    edge_res = table.get_item(Key={"f_edge": f_edge})["Item"]
    print(edge_res)
    return EdgeT(type(schema), BaseSchema, EdgeRelationship(edge_res["relationship"]))


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


def meta_into_predicate(dynamodb, schema, predicate_meta):
    try:
        if predicate_meta["type"] == "uid":
            return meta_into_edge(dynamodb, schema, predicate_meta["predicate"])
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


def extend_schema(dynamodb, graph_client: GraphClient, schema: "BaseSchema"):
    predicate_metas = query_dgraph_type(graph_client, schema.self_type())

    for predicate_meta in predicate_metas:
        predicate = meta_into_predicate(dynamodb, schema, predicate_meta)
        if isinstance(predicate, PropType):
            schema.add_property(predicate_meta["predicate"], predicate)
        else:
            schema.add_edge(predicate_meta["predicate"], predicate, "")


def provision_master_graph(
    master_graph_client: GraphClient, schemas: List["BaseSchema"]
) -> None:
    mg_schema_str = format_schemas(schemas)
    set_schema(master_graph_client, mg_schema_str)


def provision_mg(mclient) -> None:
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

    dynamodb = get_dynamodb_client()
    for schema in schemas:
        store_schema(dynamodb, schema)

    for schema in schemas:
        schema.init_reverse()

    for schema in schemas:
        extend_schema(dynamodb, mclient, schema)

    provision_master_graph(mclient, schemas)

    for schema in schemas:
        store_schema(dynamodb, schema)


def get_dynamodb_client() -> "Any":
    return boto3.resource(
        "dynamodb",
        endpoint_url="http://dynamodb:8000",
        region_name="us-west-2",
        aws_access_key_id="dummy_cred_aws_access_key_id",
        aws_secret_access_key="dummy_cred_aws_secret_access_key",
    )


def store_schema(dynamodb, schema: "Schema"):
    table = dynamodb.Table("local-grapl-grapl_schema_table")
    for f_edge, (edge_t, r_edge) in schema.get_edges().items():
        if not (f_edge and r_edge):
            continue
        table.put_item(
            Item={
                "f_edge": r_edge,
                "r_edge": f_edge,
                "relationship": int(edge_t.rel.reverse()),
            }
        )


if __name__ == "__main__":
    time.sleep(5)
    local_dg_provision_client = DgraphClient(DgraphClientStub(f"localhost:9080"))

    LOGGER.debug("Provisioning graph database")

    for i in range(0, 150):
        try:
            drop_all(local_dg_provision_client)
            break
        except Exception as e:
            time.sleep(2)
            if i > 20:
                LOGGER.error("Failed to drop", e)

    mg_succ = False
    for i in range(0, 150):
        try:
            if not mg_succ:
                time.sleep(1)
                provision_mg(
                    local_dg_provision_client,
                )
                mg_succ = True
                print("Provisioned mastergraph")
                break
        except Exception as e:
            if i > 10:
                LOGGER.error(f"mg provision failed with: {e}")

    print("Completed provisioning")
