from __future__ import annotations

import logging
import os
import sys
from hashlib import pbkdf2_hmac, sha256
from typing import TYPE_CHECKING, Any

import boto3
from grapl_analyzerlib.prelude import (
    AssetSchema,
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
from grapl_analyzerlib.provision import provision_common
from grapl_common.env_helpers import (
    DynamoDBResourceFactory,
    SecretsManagerClientFactory,
)

if TYPE_CHECKING:
    from mypy_boto3_dynamodb import DynamoDBServiceResource
    from mypy_boto3_secretsmanager import Client as SecretsmanagerClient

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(os.getenv("GRAPL_LOG_LEVEL", "INFO"))
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

DEPLOYMENT_NAME = os.environ["DEPLOYMENT_NAME"]
GRAPL_TEST_USER_NAME = os.environ["GRAPL_TEST_USER_NAME"]
GRAPL_SCHEMA_TABLE = os.environ["GRAPL_SCHEMA_TABLE"]
GRAPL_SCHEMA_PROPERTIES_TABLE = os.environ["GRAPL_SCHEMA_PROPERTIES_TABLE"]


def _provision_graph(
    graph_client: GraphClient, dynamodb: DynamoDBServiceResource
) -> None:
    schema_table = dynamodb.Table(GRAPL_SCHEMA_TABLE)
    schema_properties_table = dynamodb.Table(GRAPL_SCHEMA_PROPERTIES_TABLE)
    schemas = [
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
    ]

    for schema in schemas:
        schema.init_reverse()

    for schema in schemas:
        provision_common.extend_schema(schema_table, graph_client, schema)

    schema_str = provision_common.format_schemas(schemas)
    provision_common.set_schema(graph_client, schema_str)

    for schema in schemas:
        provision_common.store_schema(schema_table, schema)
        provision_common.store_schema_properties(schema_properties_table, schema)


def _hash_password(cleartext: bytes, salt: bytes) -> str:
    hashed = sha256(cleartext).digest()
    return pbkdf2_hmac("sha256", hashed, salt, 512000).hex()


def _create_user(
    dynamodb: DynamoDBServiceResource, username: str, cleartext: str
) -> None:
    assert cleartext
    table = dynamodb.Table(os.environ["GRAPL_USER_AUTH_TABLE"])

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
) -> str:
    return secretsmanager.get_secret_value(
        SecretId=f"{deployment_name}-TestUserPassword"
    )["SecretString"]


def provision(event: Any = None, context: Any = None) -> None:
    LOGGER.info("provisioning grapl")

    graph_client = GraphClient()
    dynamodb = DynamoDBResourceFactory(boto3).from_env()
    secretsmanager = SecretsManagerClientFactory(boto3).from_env()

    LOGGER.info("provisioning graph")
    _provision_graph(graph_client=graph_client, dynamodb=dynamodb)
    LOGGER.info("provisioned graph")

    LOGGER.info("retrieving test user password")
    password = _retrieve_test_user_password(secretsmanager, DEPLOYMENT_NAME)
    LOGGER.info("retrieved test user password")

    LOGGER.info("creating test user")
    _create_user(dynamodb=dynamodb, username=GRAPL_TEST_USER_NAME, cleartext=password)
    LOGGER.info("created test user")
