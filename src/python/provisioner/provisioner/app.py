from __future__ import annotations

import os
from typing import TYPE_CHECKING
from uuid import uuid4

import boto3
from argon2 import PasswordHasher
from grapl_common.env_helpers import (
    DynamoDBResourceFactory,
    SecretsManagerClientFactory,
)
from grapl_common.grapl_tracer import get_tracer
from grapl_common.logger import get_structlogger
from grapl_common.test_user_creds import get_test_user_creds
from python_proto.api.scylla_provisioner.v1beta1.client import ScyllaProvisionerClient
from python_proto.client import GrpcClientConfig
from python_proto.common import Uuid

if TYPE_CHECKING:
    from mypy_boto3_dynamodb import DynamoDBServiceResource  # pants: no-infer-dep
    from mypy_boto3_secretsmanager import (
        Client as SecretsmanagerClient,  # pants: no-infer-dep
    )

LOGGER = get_structlogger()

GRAPL_SCHEMA_TABLE = os.environ["GRAPL_SCHEMA_TABLE"]
GRAPL_SCHEMA_PROPERTIES_TABLE = os.environ["GRAPL_SCHEMA_PROPERTIES_TABLE"]

TRACER = get_tracer(service_name="provisioner", module_name=__name__)


def _create_user(
    dynamodb: DynamoDBServiceResource,
    username: str,
    cleartext: str,
    role: str,
    org: str,
) -> None:
    assert cleartext
    table = dynamodb.Table(os.environ["GRAPL_USER_AUTH_TABLE"])

    password_hasher = PasswordHasher(time_cost=2, memory_cost=102400, parallelism=8)
    password_hash = password_hasher.hash(cleartext)

    table.put_item(
        Item={
            "username": username,
            "password_hash": password_hash,
            "grapl_role": role,
            "organization_id": org,
        }
    )


# TODO: Replace with passing in the password ID verbatim
def _retrieve_test_user_password(
    secretsmanager: SecretsmanagerClient, deployment_name: str
) -> str:
    return secretsmanager.get_secret_value(
        SecretId=f"{deployment_name}-TestUserPassword"
    )["SecretString"]


def provision_scylla() -> None:
    LOGGER.info("provisioning scylla")
    scylla_provisioner_client = ScyllaProvisionerClient.connect(
        client_config=GrpcClientConfig(
            address=os.environ["SCYLLA_PROVISIONER_CLIENT_ADDRESS"],
        )
    )

    # scylla_provisioner's API takes a tenant_id but it doesn't use it
    # anymore
    arbitrary_tenant_id = uuid4()
    scylla_provisioner_client.provision_graph_for_tenant(
        tenant_id=Uuid.from_uuid(arbitrary_tenant_id)
    )
    LOGGER.info("provisioned scylla")


def provision() -> None:
    LOGGER.info("provisioning grapl")
    with TRACER.start_as_current_span(__name__):

        dynamodb = DynamoDBResourceFactory(boto3).from_env()
        secretsmanager = SecretsManagerClientFactory(boto3).from_env()

        username, password = get_test_user_creds()

        LOGGER.info("creating test user")
        _create_user(
            dynamodb=dynamodb,
            username=username,
            cleartext=password,
            role="owner",
            org="00000000-0000-0000-0000-000000000000",
        )
        LOGGER.info("created test user")

        provision_scylla()
