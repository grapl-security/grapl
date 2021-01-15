import json
import logging
import os
import sys
import threading
import time
from hashlib import pbkdf2_hmac, sha256
from typing import List
from uuid import uuid4

import boto3
import botocore
import pydgraph

from grapl_analyzerlib.grapl_client import GraphClient, MasterGraphClient
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

GRAPL_LOG_LEVEL = os.getenv("GRAPL_LOG_LEVEL")
LEVEL = "ERROR" if GRAPL_LOG_LEVEL is None else GRAPL_LOG_LEVEL
LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(LEVEL)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))


def create_secret(secretsmanager):
    secretsmanager.create_secret(
        Name="JWT_SECRET_ID",
        SecretString=str(uuid4()),
    )


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

    dynamodb = boto3.resource(
        "dynamodb",
        region_name="us-west-2",
        endpoint_url="http://dynamodb:8000",
        aws_access_key_id="dummy_cred_aws_access_key_id",
        aws_secret_access_key="dummy_cred_aws_secret_access_key",
    )

    table = dynamodb.Table("local-grapl-grapl_schema_table")
    for schema in schemas:
        store_schema(table, schema)


BUCKET_PREFIX = "local-grapl"

services = (
    "sysmon-graph-generator",
    "osquery-graph-generator",
    "generic-graph-generator",
    "node-identifier",
    "graph-merger",
    "analyzer-dispatcher",
    "analyzer-executor",
    "engagement-creator",
)

buckets = (
    BUCKET_PREFIX + "-sysmon-log-bucket",
    BUCKET_PREFIX + "-osquery-log-bucket",
    BUCKET_PREFIX + "-unid-subgraphs-generated-bucket",
    BUCKET_PREFIX + "-subgraphs-generated-bucket",
    BUCKET_PREFIX + "-subgraphs-merged-bucket",
    BUCKET_PREFIX + "-analyzer-dispatched-bucket",
    BUCKET_PREFIX + "-analyzers-bucket",
    BUCKET_PREFIX + "-analyzer-matched-subgraphs-bucket",
    BUCKET_PREFIX + "-model-plugins-bucket",
)


def provision_sqs(sqs, service_name: str) -> None:
    redrive_queue = sqs.create_queue(
        QueueName="grapl-%s-retry-queue" % service_name,
        Attributes={"MessageRetentionPeriod": "86400"},
    )

    redrive_url = redrive_queue["QueueUrl"]
    LOGGER.debug(f"Provisioned {service_name} retry queue at " + redrive_url)

    redrive_arn = sqs.get_queue_attributes(
        QueueUrl=redrive_url, AttributeNames=["QueueArn"]
    )["Attributes"]["QueueArn"]

    redrive_policy = {
        "deadLetterTargetArn": redrive_arn,
        "maxReceiveCount": "10",
    }

    queue = sqs.create_queue(
        QueueName="grapl-%s-queue" % service_name,
    )

    sqs.set_queue_attributes(
        QueueUrl=queue["QueueUrl"],
        Attributes={"RedrivePolicy": json.dumps(redrive_policy)},
    )
    LOGGER.debug(f"Provisioned {service_name} queue at " + queue["QueueUrl"])

    sqs.purge_queue(QueueUrl=queue["QueueUrl"])
    sqs.purge_queue(QueueUrl=redrive_queue["QueueUrl"])


def provision_bucket(s3, bucket_name: str) -> None:
    s3.create_bucket(Bucket=bucket_name)
    LOGGER.debug(bucket_name)


def bucket_provision_loop() -> None:
    s3_succ = {bucket for bucket in buckets}
    s3 = None
    for i in range(0, 150):
        try:
            s3 = s3 or boto3.client(
                "s3",
                endpoint_url="http://s3:9000",
                aws_access_key_id="minioadmin",
                aws_secret_access_key="minioadmin",
                region_name="us-east-1",
            )
        except Exception as e:
            if i > 10:
                LOGGER.debug("failed to connect to sqs or s3", e)
            continue

        for bucket in buckets:
            if bucket in s3_succ:
                try:
                    provision_bucket(s3, bucket)
                    s3_succ.discard(bucket)
                except Exception as e:
                    if "BucketAlreadyOwnedByYou" in str(e):
                        s3_succ.discard(bucket)
                        continue

                    if i > 10:
                        LOGGER.debug(e)
                    time.sleep(1)

        if not s3_succ:
            return

    raise Exception("Failed to provision s3")


def hash_password(cleartext, salt) -> str:
    hashed = sha256(cleartext).digest()
    return pbkdf2_hmac("sha256", hashed, salt, 512000).hex()


def create_user(username, cleartext):
    assert cleartext
    dynamodb = boto3.resource(
        "dynamodb",
        region_name="us-west-2",
        endpoint_url="http://dynamodb:8000",
        aws_access_key_id="dummy_cred_aws_access_key_id",
        aws_secret_access_key="dummy_cred_aws_secret_access_key",
    )
    table = dynamodb.Table("local-grapl-user_auth_table")

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


def sqs_provision_loop() -> None:
    sqs_succ = {service for service in services}
    sqs = None
    for i in range(0, 150):
        try:
            sqs = sqs or boto3.client(
                "sqs",
                region_name="us-east-1",
                endpoint_url="http://sqs.us-east-1.amazonaws.com:9324",
                aws_access_key_id="dummy_cred_aws_access_key_id",
                aws_secret_access_key="dummy_cred_aws_secret_access_key",
            )
        except Exception as e:
            if i > 50:
                LOGGER.error("failed to connect to sqs or s3", e)
            else:
                LOGGER.debug("failed to connect to sqs or s3", e)

            time.sleep(1)
            continue

        for service in services:
            if service in sqs_succ:
                try:
                    provision_sqs(sqs, service)
                    sqs_succ.discard(service)
                except Exception as e:
                    if i > 10:
                        LOGGER.error(e)
                    time.sleep(1)
        if not sqs_succ:
            return

    raise Exception("Failed to provision sqs")


if __name__ == "__main__":
    time.sleep(5)
    local_dg_provision_client = MasterGraphClient()

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

    sqs_t = threading.Thread(target=sqs_provision_loop)
    s3_t = threading.Thread(target=bucket_provision_loop)

    sqs_t.start()
    s3_t.start()

    LOGGER.info("Starting to provision master graph")
    for i in range(0, 150):
        try:
            if not mg_succ:
                time.sleep(1)
                provision_mg(
                    local_dg_provision_client,
                )
                mg_succ = True
                LOGGER.info("Provisioned master graph")
                break
        except Exception as e:
            if i > 10:
                LOGGER.error("mg provision failed with: ", e)

    LOGGER.info("Starting to provision Secrets Manager")
    for i in range(0, 150):
        try:
            client = boto3.client(
                service_name="secretsmanager",
                region_name="us-east-1",
                endpoint_url="http://secretsmanager.us-east-1.amazonaws.com:4584",
                aws_access_key_id="dummy_cred_aws_access_key_id",
                aws_secret_access_key="dummy_cred_aws_secret_access_key",
            )
            create_secret(client)
            LOGGER.info("Done provisioning Secrets Manager")
            break
        except botocore.exceptions.ClientError as e:
            if "ResourceExistsException" in e.__class__.__name__:
                break
            if i >= 50:
                LOGGER.debug(e)
        except Exception as e:
            if i >= 50:
                LOGGER.error(e)
            time.sleep(1)

    LOGGER.info("Starting to provision Grapl user")
    for i in range(0, 150):
        try:
            create_user("grapluser", "graplpassword")
            LOGGER.info("Done provisioning Grapl user")
            break
        except Exception as e:
            if i >= 50:
                LOGGER.error(e)
            time.sleep(1)

    LOGGER.info("Ensuring S3/SQS completed...")
    sqs_t.join(timeout=300)
    s3_t.join(timeout=300)
    LOGGER.info("S3/SQS completed")

    LOGGER.info("Completed provisioning")
