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
        ("access_session_token", str),
    ),
)


def _client_get(
    client_create_fn: Callable[..., Any],
    params: ClientGetParams,
    region: Optional[str] = None,
    config: Optional[Config] = None,
) -> Any:
    """
    :param client_create_fn: the `boto3.client` or `boto3.resource` function
    """
    which_service = params.boto3_client_name
    endpoint_url = os.getenv(params.endpoint_url_key)
    access_key_id = os.getenv(params.access_key_id_key)
    access_key_secret = os.getenv(params.access_key_secret_key)
    access_session_token = os.getenv(params.access_session_token)

    # AWS_REGION is Fargate-specific, most AWS stuff uses AWS_DEFAULT_REGION.
    if not region:
        region = os.getenv("AWS_REGION") or os.getenv("AWS_DEFAULT_REGION")
        if not region:
            raise FromEnvException("Please set AWS_REGION= or AWS_DEFAULT_REGION=")

    # Unlike Rust FromEnv, we rely on boto3's built in region handling.

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
        logging.info("Creating a prod client")
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
    endpoint_url_key="SQS_ENDPOINT",
    access_key_id_key="SQS_ACCESS_KEY_ID",
    access_key_secret_key="SQS_ACCESS_KEY_SECRET",
    access_session_token="SQS_ACCESS_SESSION_TOKEN",
)


class SQSClientFactory(FromEnv["SQSClient"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(
        self, region: Optional[str] = None, config: Optional[Config] = None
    ) -> SQSClient:
        client: SQSClient = _client_get(
            self.client_create_fn, _SQSParams, region=region, config=config
        )
        return client


_SNSParams = ClientGetParams(
    boto3_client_name="sns",
    endpoint_url_key="SNS_ENDPOINT",
    access_key_id_key="SNS_ACCESS_KEY_ID",
    access_key_secret_key="SNS_ACCESS_KEY_SECRET",
    access_session_token="SNS_ACCESS_SESSION_TOKEN",
)


class SNSClientFactory(FromEnv["SNSClient"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(
        self, region: Optional[str] = None, config: Optional[Config] = None
    ) -> SNSClient:
        client: SNSClient = _client_get(
            self.client_create_fn, _SNSParams, region=region, config=config
        )
        return client


_EC2Params = ClientGetParams(
    boto3_client_name="ec2",
    endpoint_url_key="EC2_ENDPOINT",
    access_key_id_key="EC2_ACCESS_KEY_ID",
    access_key_secret_key="EC2_ACCESS_KEY_SECRET",
    access_session_token="EC2_ACCESS_SESSION_TOKEN",
)


class EC2ResourceFactory(FromEnv["EC2ServiceResource"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.resource

    def from_env(
        self, region: Optional[str] = None, config: Optional[Config] = None
    ) -> EC2ServiceResource:
        client: EC2ServiceResource = _client_get(
            self.client_create_fn, _EC2Params, region=region, config=config
        )
        return client


_SSMParams = ClientGetParams(
    boto3_client_name="ssm",
    endpoint_url_key="SSM_ENDPOINT",
    access_key_id_key="SSM_ACCESS_KEY_ID",
    access_key_secret_key="SSM_ACCESS_KEY_SECRET",
    access_session_token="SSM_ACCESS_SESSION_TOKEN",
)


class SSMClientFactory(FromEnv["SSMClient"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(
        self, region: Optional[str] = None, config: Optional[Config] = None
    ) -> SSMClient:
        client: SSMClient = _client_get(
            self.client_create_fn, _SSMParams, region=region, config=config
        )
        return client


_CloudWatchParams = ClientGetParams(
    boto3_client_name="cloudwatch",
    endpoint_url_key="CLOUDWATCH_ENDPOINT",
    access_key_id_key="CLOUDWATCH_ACCESS_KEY_ID",
    access_key_secret_key="CLOUDWATCH_ACCESS_KEY_SECRET",
    access_session_token="CLOUDWATCH_ACCESS_SESSION_TOKEN",
)


class CloudWatchClientFactory(FromEnv["CloudWatchClient"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(
        self, region: Optional[str] = None, config: Optional[Config] = None
    ) -> CloudWatchClient:
        client: CloudWatchClient = _client_get(
            self.client_create_fn, _CloudWatchParams, region=region, config=config
        )
        return client


_LambdaParams = ClientGetParams(
    boto3_client_name="lambda",
    endpoint_url_key="LAMBDA_ENDPOINT",
    access_key_id_key="LAMBDA_ACCESS_KEY_ID",
    access_key_secret_key="LAMBDA_ACCESS_KEY_SECRET",
    access_session_token="LAMBDA_ACCESS_SESSION_TOKEN",
)


class LambdaClientFactory(FromEnv["LambdaClient"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(
        self, region: Optional[str] = None, config: Optional[Config] = None
    ) -> LambdaClient:
        client: LambdaClient = _client_get(
            self.client_create_fn, _LambdaParams, region=region, config=config
        )
        return client


_Route53Params = ClientGetParams(
    boto3_client_name="route53",
    endpoint_url_key="ROUTE53_ENDPOINT",
    access_key_id_key="ROUTE53_ACCESS_KEY_ID",
    access_key_secret_key="ROUTE53_ACCESS_KEY_SECRET",
    access_session_token="ROUTE53_ACCESS_SESSION_TOKEN",
)


class Route53ClientFactory(FromEnv["Route53Client"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(
        self, region: Optional[str] = None, config: Optional[Config] = None
    ) -> Route53Client:
        client: Route53Client = _client_get(
            self.client_create_fn, _Route53Params, region=region, config=config
        )
        return client


_S3Params = ClientGetParams(
    boto3_client_name="s3",
    endpoint_url_key="S3_ENDPOINT",
    access_key_id_key="S3_ACCESS_KEY_ID",
    access_key_secret_key="S3_ACCESS_KEY_SECRET",
    access_session_token="S3_ACCESS_SESSION_TOKEN",
)


class S3ClientFactory(FromEnv["S3Client"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(
        self, region: Optional[str] = None, config: Optional[Config] = None
    ) -> S3Client:
        client: S3Client = _client_get(
            self.client_create_fn, _S3Params, region=region, config=config
        )
        return client


class S3ResourceFactory(FromEnv["S3ServiceResource"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.resource

    def from_env(
        self, region: Optional[str] = None, config: Optional[Config] = None
    ) -> S3ServiceResource:
        client: S3ServiceResource = _client_get(
            self.client_create_fn, _S3Params, region=region, config=config
        )
        return client


_DynamoDBParams = ClientGetParams(
    boto3_client_name="dynamodb",
    endpoint_url_key="DYNAMODB_ENDPOINT",
    access_key_id_key="DYNAMODB_ACCESS_KEY_ID",
    access_key_secret_key="DYNAMODB_ACCESS_KEY_SECRET",
    access_session_token="DYNAMODB_ACCESS_SESSION_TOKEN",
)


class DynamoDBResourceFactory(FromEnv["DynamoDBServiceResource"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.resource

    def from_env(
        self, region: Optional[str] = None, config: Optional[Config] = None
    ) -> DynamoDBServiceResource:
        client: DynamoDBServiceResource = _client_get(
            self.client_create_fn, _DynamoDBParams, region=region, config=config
        )
        return client


class DynamoDBClientFactory(FromEnv["DynamoDBClient"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(self, region: Optional[str] = None) -> DynamoDBClient:
        client: DynamoDBClient = _client_get(
            self.client_create_fn, _DynamoDBParams, region=region
        )
        return client


_SecretsManagerParams = ClientGetParams(
    boto3_client_name="secretsmanager",
    endpoint_url_key="SECRETSMANAGER_ENDPOINT",
    access_key_id_key="SECRETSMANAGER_ACCESS_KEY_ID",
    access_key_secret_key="SECRETSMANAGER_ACCESS_KEY_SECRET",
    access_session_token="SECRETSMANAGER_ACCESS_SESSION_TOKEN",
)


class SecretsManagerClientFactory(FromEnv["SecretsManagerClient"]):
    def __init__(self, boto3_module: Any):
        self.client_create_fn = boto3_module.client

    def from_env(
        self, region: Optional[str] = None, config: Optional[Config] = None
    ) -> SecretsManagerClient:
        client: SecretsManagerClient = _client_get(
            self.client_create_fn, _SecretsManagerParams, region=region, config=config
        )
        return client


def get_deployment_name() -> str:
    return os.environ["DEPLOYMENT_NAME"]
