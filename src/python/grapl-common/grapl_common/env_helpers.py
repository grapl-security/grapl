from __future__ import annotations

import logging
import os
from typing import TYPE_CHECKING, Any, Callable, NamedTuple, Optional, TypeVar

from botocore.client import Config
from typing_extensions import Protocol

if TYPE_CHECKING:
    from mypy_boto3_cloudwatch import CloudWatchClient
    from mypy_boto3_dynamodb import DynamoDBClient, DynamoDBServiceResource
    from mypy_boto3_ec2 import EC2ServiceResource
    from mypy_boto3_lambda import LambdaClient
    from mypy_boto3_route53 import Route53Client
    from mypy_boto3_s3 import S3Client, S3ServiceResource
    from mypy_boto3_secretsmanager import SecretsManagerClient
    from mypy_boto3_sns import SNSClient
    from mypy_boto3_sqs import SQSClient
    from mypy_boto3_ssm import SSMClient


T = TypeVar("T", covariant=True)


class FromEnv(Protocol[T]):
    def from_env(self, config: Optional[Config] = None) -> T:
        pass


class FromEnvException(Exception):
    pass


ClientGetParams = NamedTuple(
    "ClientGetParams",
    (("boto3_client_name", str),),  # e.g. "s3" or "sqs"
)


def _client_get(
    client_create_fn: Callable[..., Any],
    params: ClientGetParams,
    config: Optional[Config] = None,
) -> Any:
    """
    :param client_create_fn: the `boto3.client` or `boto3.resource` function
    """
    which_service = params.boto3_client_name
    endpoint_url = os.getenv("GRAPL_AWS_ENDPOINT")
    access_key_id = os.getenv("GRAPL_AWS_ACCESS_KEY_ID")
    access_key_secret = os.getenv("GRAPL_AWS_ACCESS_KEY_SECRET")
    access_session_token = os.getenv("GRAPL_AWS_ACCESS_SESSION_TOKEN")

    # determine the aws region
    if config is not None and config.region_name is not None:
        # prefer config's region if set
        region = config.region_name
    else:
        region = os.getenv("AWS_DEFAULT_REGION") or os.getenv("AWS_REGION")

    if not region:
        raise FromEnvException(
            "Please set AWS_REGION, AWS_DEFAULT_REGION, or config.region_name"
        )

    if _running_in_localstack():
        localstack_config = Config(
            read_timeout=120,
        )
        config = (config or Config()).merge(localstack_config)

        return _localstack_client(
            client_create_fn, params, region=region, config=config
        )
    elif all((endpoint_url, access_key_id, access_key_secret)):
        # Local, all are passed in from docker-compose.yml
        logging.info(f"Creating a local client for {which_service}")
        return client_create_fn(
            params.boto3_client_name,
            endpoint_url=endpoint_url,
            aws_access_key_id=access_key_id,
            aws_secret_access_key=access_key_secret,
            aws_session_token=access_session_token,
            region_name=region,
            config=config,
        )
    elif endpoint_url and not any((access_key_id, access_key_secret)):
        # Local or AWS doing cross-region stuff
        return client_create_fn(
            params.boto3_client_name,
            endpoint_url=endpoint_url,
            region_name=region,
            config=config,
        )
    elif not any((endpoint_url, access_key_id, access_key_secret)):
        # AWS
        logging.info(f"Creating a prod client for {which_service}")
        return client_create_fn(
            params.boto3_client_name,
            region_name=region,
            config=config,
        )
    else:
        raise FromEnvException(
            f"You specified access key but not endpoint for {params.boto3_client_name}?"
        )


def _running_in_localstack() -> bool:
    """Detects whether or not code is running in Localstack.

    When running lambda functions in Localstack, the
    `LOCALSTACK_HOSTNAME` environment variable will be set, allowing
    us to compose an appropriate endpoint at which the lambda can
    interact with other Localstack-hosted AWS services.

    Needless to say, this should not be present in the environment of
    a lambda actually running in AWS.

    """
    return "LOCALSTACK_HOSTNAME" in os.environ


def _localstack_client(
    client_create_fn: Callable[..., Any],
    params: ClientGetParams,
    region: Optional[str],
    config: Optional[Config],
) -> Any:
    """Create a boto3 client for interacting with AWS services running in
    Localstack from a lambda function also running in Localstack.


    Localstack provides LOCALSTACK_HOSTNAME and EDGE_PORT for creating
    a proper endpoint value. In addition, appropriate values for
    AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, and AWS_REGION are also
    provided.

    See the _running_in_localstack function, as well.
    """
    service = params.boto3_client_name
    logging.info("Creating a {service} client for Localstack!")
    return client_create_fn(
        service,
        endpoint_url=f"http://{os.environ['LOCALSTACK_HOSTNAME']}:{os.environ['EDGE_PORT']}",
        region_name=region,
        config=config,
    )


_SQSParams = ClientGetParams(
    boto3_client_name="sqs",
)


class SQSClientFactory(FromEnv["SQSClient"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(self, config: Optional[Config] = None) -> SQSClient:
        client: SQSClient = _client_get(
            self.client_create_fn, _SQSParams, config=config
        )
        return client


_SNSParams = ClientGetParams(
    boto3_client_name="sns",
)


class SNSClientFactory(FromEnv["SNSClient"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(self, config: Optional[Config] = None) -> SNSClient:
        client: SNSClient = _client_get(
            self.client_create_fn, _SNSParams, config=config
        )
        return client


_EC2Params = ClientGetParams(
    boto3_client_name="ec2",
)


class EC2ResourceFactory(FromEnv["EC2ServiceResource"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.resource

    def from_env(self, config: Optional[Config] = None) -> EC2ServiceResource:
        client: EC2ServiceResource = _client_get(
            self.client_create_fn, _EC2Params, config=config
        )
        return client


_SSMParams = ClientGetParams(
    boto3_client_name="ssm",
)


class SSMClientFactory(FromEnv["SSMClient"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(self, config: Optional[Config] = None) -> SSMClient:
        client: SSMClient = _client_get(
            self.client_create_fn, _SSMParams, config=config
        )
        return client


_CloudWatchParams = ClientGetParams(
    boto3_client_name="cloudwatch",
)


class CloudWatchClientFactory(FromEnv["CloudWatchClient"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(self, config: Optional[Config] = None) -> CloudWatchClient:
        client: CloudWatchClient = _client_get(
            self.client_create_fn, _CloudWatchParams, config=config
        )
        return client


_LambdaParams = ClientGetParams(
    boto3_client_name="lambda",
)


class LambdaClientFactory(FromEnv["LambdaClient"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(self, config: Optional[Config] = None) -> LambdaClient:
        client: LambdaClient = _client_get(
            self.client_create_fn, _LambdaParams, config=config
        )
        return client


_Route53Params = ClientGetParams(
    boto3_client_name="route53",
)


class Route53ClientFactory(FromEnv["Route53Client"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(self, config: Optional[Config] = None) -> Route53Client:
        client: Route53Client = _client_get(
            self.client_create_fn, _Route53Params, config=config
        )
        return client


_S3Params = ClientGetParams(
    boto3_client_name="s3",
)


class S3ClientFactory(FromEnv["S3Client"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(self, config: Optional[Config] = None) -> S3Client:
        client: S3Client = _client_get(self.client_create_fn, _S3Params, config=config)
        return client


class S3ResourceFactory(FromEnv["S3ServiceResource"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.resource

    def from_env(self, config: Optional[Config] = None) -> S3ServiceResource:
        client: S3ServiceResource = _client_get(
            self.client_create_fn, _S3Params, config=config
        )
        return client


_DynamoDBParams = ClientGetParams(
    boto3_client_name="dynamodb",
)


class DynamoDBResourceFactory(FromEnv["DynamoDBServiceResource"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.resource

    def from_env(self, config: Optional[Config] = None) -> DynamoDBServiceResource:
        client: DynamoDBServiceResource = _client_get(
            self.client_create_fn, _DynamoDBParams, config=config
        )
        return client


class DynamoDBClientFactory(FromEnv["DynamoDBClient"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(self, config: Optional[Config] = None) -> DynamoDBClient:
        client: DynamoDBClient = _client_get(
            self.client_create_fn, _DynamoDBParams, config=config
        )
        return client


_SecretsManagerParams = ClientGetParams(
    boto3_client_name="secretsmanager",
)


class SecretsManagerClientFactory(FromEnv["SecretsManagerClient"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(self, config: Optional[Config] = None) -> SecretsManagerClient:
        client: SecretsManagerClient = _client_get(
            self.client_create_fn, _SecretsManagerParams, config=config
        )
        return client


def get_deployment_name() -> str:
    return os.environ["DEPLOYMENT_NAME"]
