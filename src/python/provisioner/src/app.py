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

if TYPE_CHECKING:
    from mypy_boto3_dynamodb import DynamoDBServiceResource
    from mypy_boto3_secretsmanager import Client as SecretsmanagerClient

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(os.getenv("GRAPL_LOG_LEVEL", "INFO"))
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

DEPLOYMENT_NAME = os.environ["DEPLOYMENT_NAME"]
GRAPL_TEST_USER_NAME = os.environ["GRAPL_TEST_USER_NAME"]


def _provision_graph(
    graph_client: GraphClient, dynamodb: DynamoDBServiceResource
) -> None:
    schema_table = provision_common.get_schema_table(
        dynamodb, deployment_name=DEPLOYMENT_NAME
    )
    schema_properties_table = provision_common.get_schema_properties_table(
        dynamodb, deployment_name=DEPLOYMENT_NAME
    )
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
) -> str:
    return secretsmanager.get_secret_value(
        SecretId=f"{deployment_name}-TestUserPassword"
    )["SecretString"]


def _validate_environment():
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


def provision(event: Any = None, context: Any = None):
    _validate_environment()
    graph_client = GraphClient()
    dynamodb: DynamoDBServiceResource = boto3.resource("dynamodb")
    secretsmanager: SecretsmanagerClient = boto3.client("secretsmanager")

    _provision_graph(graph_client=graph_client, dynamodb=dynamodb)

    password = _retrieve_test_user_password(secretsmanager, DEPLOYMENT_NAME)
    _create_user(dynamodb=dynamodb, username=GRAPL_TEST_USER_NAME, cleartext=password)


if __name__ == "__main__":
    provision()
