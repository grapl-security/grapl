import dataclasses
from typing import Dict, List, Union

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

    def as_dict(self) -> Dict[str, Union[str, int, bool, List[int]]]:
        dataclasses.asdict(self)


@dataclasses.dataclass
class PortConfig:
    protocol: str
    port: int

    def as_dict(self) -> Dict[str, Union[str, int]]:
        dataclasses.asdict(self)


@dataclasses.dataclass
class TableConfig:
    table: str
    write: bool

    def as_dict(self) -> Dict[str, Union[str, bool]]:
        dataclasses.asdict(self)


@dataclasses.dataclass
class SecretConfig:
    SecretId: str
    VersionId: str = None
    VersionStage: str = None

    def as_dict(self) -> Dict[str, str]:
        dataclasses.asdict(self)


@dataclasses.dataclass
class AnalyzerConfig:
    requires_external_internet: List[PortConfig] = None
    requires_dynamodb: List[TableConfig] = None
    requires_graph: bool = False
    requires_secrets: List[SecretConfig] = None

    def as_dict(self) -> Dict[str, Union[str, bool, int, List[Dict[str, Union[str, int]]]]]:
        dataclasses.asdict(self)


@dataclasses.dataclass
class AnalyzerDeployment:
    analyzer_id: str
    analyzer_version: int
    s3_key: str
    currently_deployed: bool
    last_deployed_time: int = None
    analyzer_configuration: AnalyzerConfig = None

    def as_dict(self) -> Dict[str, Union[str, int, bool]]:
        dataclasses.asdict(self)


@dataclasses.dataclass
class CreateAnalyzerResponse:
    analyzer_id: str
    analyzer_version: int
    s3_key: str

    def as_dict(self) -> Dict[str, Union[str, int]]:
        dataclasses.asdict(self)


def _create_analyzer(
    dynamodb_client: dynamodb.DynamoDBServiceResource,
) -> CreateAnalyzerResponse:
    analyzer = Analyzer()
    analyzers_table = dynamodb_client.Table("Analyzers")
    analyzers_table.put_item(Item=)


@app.route("/1/analyzers", methods=["POST"])
def create_analyzer():
    return _create_analyzer().as_dict()


# The view function above will return {"hello": "world"}
# whenever you make an HTTP GET request to '/'.
#
# Here are a few more examples:
#
# @app.route('/hello/{name}')
# def hello_name(name):
#    # '/hello/james' -> {"hello": "james"}
#    return {'hello': name}
#
# @app.route('/users', methods=['POST'])
# def create_user():
#     # This is the JSON body the user sent in their POST request.
#     user_as_json = app.current_request.json_body
#     # We'll echo the json body back to the user in a 'user' key.
#     return {'user': user_as_json}
#
# See the README documentation for more examples.
#
