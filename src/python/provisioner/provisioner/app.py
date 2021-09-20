from __future__ import annotations

import logging
import os
import sys
from typing import TYPE_CHECKING, Any

import boto3
from argon2 import PasswordHasher
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


def _create_user(
    dynamodb: DynamoDBServiceResource, username: str, cleartext: str, role: str
) -> None:
    assert cleartext
    table = dynamodb.Table(os.environ["GRAPL_USER_AUTH_TABLE"])

    password_hasher = PasswordHasher(time_cost=2, memory_cost=102400, parallelism=8)
    password_hash = password_hasher.hash(cleartext)

    table.put_item(
        Item={"username": username, "password_hash": password_hash, "grapl_role": role}
    )


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
    _create_user(
        dynamodb=dynamodb,
        username=GRAPL_TEST_USER_NAME,
        cleartext=password,
        role="owner",
    )
    LOGGER.info("created test user")
