from __future__ import annotations

import logging
import os
from typing import TYPE_CHECKING, Any, Callable, NamedTuple, TypeVar

from typing_extensions import Protocol

if TYPE_CHECKING:
    from mypy_boto3_dynamodb import DynamoDBClient, DynamoDBServiceResource
    from mypy_boto3_s3 import S3Client, S3ServiceResource
    from mypy_boto3_secretsmanager import SecretsManagerClient
    from mypy_boto3_sqs import SQSClient

T = TypeVar("T", covariant=True)


class FromEnv(Protocol[T]):
    def from_env(self) -> T:
        pass


class FromEnvException(Exception):
    pass


ClientGetParams = NamedTuple(
    "ClientGetParams",
    (
        ("boto3_client_name", str),  # e.g. "s3" or "sqs"
        ("endpoint_url_key", str),  # e.g. "SQS_ENDPOINT"
        ("access_key_id_key", str),
        ("access_key_secret_key", str),
    ),
)


def _client_get(client_create_fn: Callable[..., Any], params: ClientGetParams) -> Any:
    """
    :param client_create_fn: the `boto3.client` or `boto3.resource` function
    """
    which_service = params.boto3_client_name
    endpoint_url = os.getenv(params.endpoint_url_key)
    access_key_id = os.getenv(params.access_key_id_key)
    access_key_secret = os.getenv(params.access_key_secret_key)

    # AWS_REGION is Fargate-specific, most AWS stuff uses AWS_DEFAULT_REGION.
    region = os.getenv("AWS_REGION") or os.getenv("AWS_DEFAULT_REGION")
    if not region:
        raise FromEnvException("Please set AWS_REGION= or AWS_DEFAULT_REGION=")

    # Not needed long term, more to help migrate to `env_helpers`.
    # Notably, when `is_local` is not set, it won't break anything.
    is_local = os.getenv("IS_LOCAL", None)

    # Unlike Rust FromEnv, we rely on boto3's built in region handling.

    if all((endpoint_url, access_key_id, access_key_secret)):
        # Local, all are passed in from docker-compose.yml
        logging.info(f"Creating a local client for {which_service}")
        assert (
            is_local != False
        ), f"You must pass in credentials for a local {which_service} client"
        return client_create_fn(
            params.boto3_client_name,
            endpoint_url=endpoint_url,
            aws_access_key_id=access_key_id,
            aws_secret_access_key=access_key_secret,
            region_name=region,
        )
    elif endpoint_url and not any((access_key_id, access_key_secret)):
        # Local or AWS doing cross-region stuff
        return client_create_fn(
            params.boto3_client_name,
            endpoint_url=endpoint_url,
            region_name=region,
        )
    elif not any((endpoint_url, access_key_id, access_key_secret)):
        # AWS
        logging.info("Creating a prod client")
        assert (
            is_local != True
        ), f"You can't pass in credentials for a prod {which_service} client"
        return client_create_fn(
            params.boto3_client_name,
            region_name=region,
        )
    else:
        raise FromEnvException(
            f"You specified access key but not endpoint for {params.boto3_client_name}?"
        )


_SQSParams = ClientGetParams(
    boto3_client_name="sqs",
    endpoint_url_key="SQS_ENDPOINT",
    access_key_id_key="SQS_ACCESS_KEY_ID",
    access_key_secret_key="SQS_ACCESS_KEY_SECRET",
)


class SQSClientFactory(FromEnv["SQSClient"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(self) -> SQSClient:
        client: SQSClient = _client_get(self.client_create_fn, _SQSParams)
        return client


_S3Params = ClientGetParams(
    boto3_client_name="s3",
    endpoint_url_key="S3_ENDPOINT",
    access_key_id_key="S3_ACCESS_KEY_ID",
    access_key_secret_key="S3_ACCESS_KEY_SECRET",
)


class S3ClientFactory(FromEnv["S3Client"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(self) -> S3Client:
        client: S3Client = _client_get(self.client_create_fn, _S3Params)
        return client


class S3ResourceFactory(FromEnv["S3ServiceResource"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.resource

    def from_env(self) -> S3ServiceResource:
        client: S3ServiceResource = _client_get(self.client_create_fn, _S3Params)
        return client


_DynamoDBParams = ClientGetParams(
    boto3_client_name="dynamodb",
    endpoint_url_key="DYNAMODB_ENDPOINT",
    access_key_id_key="DYNAMODB_ACCESS_KEY_ID",
    access_key_secret_key="DYNAMODB_ACCESS_KEY_SECRET",
)


class DynamoDBResourceFactory(FromEnv["DynamoDBServiceResource"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.resource

    def from_env(self) -> DynamoDBServiceResource:
        client: DynamoDBServiceResource = _client_get(
            self.client_create_fn, _DynamoDBParams
        )
        return client


class DynamoDBClientFactory(FromEnv["DynamoDBClient"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(self) -> DynamoDBClient:
        client: DynamoDBClient = _client_get(self.client_create_fn, _DynamoDBParams)
        return client


_SecretsManagerParams = ClientGetParams(
    boto3_client_name="secretsmanager",
    endpoint_url_key="SECRETSMANAGER_ENDPOINT",
    access_key_id_key="SECRETSMANAGER_ACCESS_KEY_ID",
    access_key_secret_key="SECRETSMANAGER_ACCESS_KEY_SECRET",
)


class SecretsManagerClientFactory(FromEnv["SecretsManagerClient"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(self) -> SecretsManagerClient:
        client: SecretsManagerClient = _client_get(
            self.client_create_fn, _SecretsManagerParams
        )
        return client
