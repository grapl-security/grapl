import dataclasses
from typing import List, Optional

import boto3
from chalice import Chalice
from mypy_boto3_dynamodb import DynamoDBServiceResource
from mypy_boto3_sqs import SQSServiceResource

app = Chalice(app_name="analyzer-deployer")


def _create_queue(queue_name: str):
    client: SQSServiceResource = boto3.resource("sqs")
    client.create_queue(QueueName=queue_name)
    pass


@dataclasses.dataclass
class Analyzer:
    analyzer_id: str
    analyzer_versions: List[int]
    analyzer_active: bool
    created_time: int
    last_update_time: int


@dataclasses.dataclass
class PortConfig:
    protocol: str
    port: int


@dataclasses.dataclass
class TableConfig:
    table: str
    write: bool


@dataclasses.dataclass
class SecretConfig:
    SecretId: str
    VersionId: Optional[str] = None
    VersionStage: Optional[str] = None


@dataclasses.dataclass
class AnalyzerConfig:
    requires_external_internet: Optional[List[PortConfig]] = None
    requires_dynamodb: Optional[List[TableConfig]] = None
    requires_graph: bool = False
    requires_secrets: Optional[List[SecretConfig]] = None


@dataclasses.dataclass
class AnalyzerDeployment:
    analyzer_id: str
    analyzer_version: int
    s3_key: str
    currently_deployed: bool
    last_deployed_time: Optional[int] = None
    analyzer_configuration: Optional[AnalyzerConfig] = None


@dataclasses.dataclass
class CreateAnalyzerResponse:
    analyzer_id: str
    analyzer_version: int
    s3_key: str


def _create_analyzer(
    dynamodb_client: DynamoDBServiceResource,
) -> CreateAnalyzerResponse:
    analyzer = Analyzer()  # type: ignore
    analyzers_table = dynamodb_client.Table("Analyzers")
    return CreateAnalyzerResponse("id", 0, "key")  # FIXME


@app.route("/1/analyzers", methods=["POST"])
def create_analyzer():
    dynamodb_client = boto3.resource("dynamodb")
    return dataclasses.asdict(_create_analyzer(dynamodb_client))
