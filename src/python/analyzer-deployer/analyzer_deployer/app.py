import dataclasses
from typing import List

import boto3

from mypy_boto3 import sqs, dynamodb

from chalice import Chalice

app = Chalice(app_name="analyzer-deployer")


def _create_queue(queue_name: str):
    client: sqs.Client = boto3.resource("sqs")
    client.create_queue()
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
    VersionId: str = None
    VersionStage: str = None


@dataclasses.dataclass
class AnalyzerConfig:
    requires_external_internet: List[PortConfig] = None
    requires_dynamodb: List[TableConfig] = None
    requires_graph: bool = False
    requires_secrets: List[SecretConfig] = None


@dataclasses.dataclass
class AnalyzerDeployment:
    analyzer_id: str
    analyzer_version: int
    s3_key: str
    currently_deployed: bool
    last_deployed_time: int = None
    analyzer_configuration: AnalyzerConfig = None


@dataclasses.dataclass
class CreateAnalyzerResponse:
    analyzer_id: str
    analyzer_version: int
    s3_key: str


def _create_analyzer(
    dynamodb_client: dynamodb.DynamoDBServiceResource,
) -> CreateAnalyzerResponse:
    analyzer = Analyzer()
    analyzers_table = dynamodb_client.Table("Analyzers")
    return CreateAnalyzerResponse("id", 0, "key")  # FIXME


@app.route("/1/analyzers", methods=["POST"])
def create_analyzer():
    dynamodb_client = boto3.resource("dynamodb")
    return dataclasses.asdict(_create_analyzer(dynamodb_client))
