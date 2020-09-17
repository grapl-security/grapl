import dataclasses
import logging
import os
import sys
import uuid

from datetime import datetime, timezone, timedelta
from typing import (
    Any,
    Dict,
    Generic,
    Iterable,
    Iterator,
    Optional,
    Tuple,
    TypeVar,
    Union,
)
from typing_extensions import Literal

import boto3

from mypy_boto3 import dynamodb, s3, sqs

from chalice import Chalice, NotFoundError

from grapl_common.time_utils import Millis, as_millis

IS_LOCAL = bool(os.environ.get("IS_LOCAL", False))

ANALYZERS_TABLE = os.getenv("ANALYZERS_TABLE")
ANALYZERS_BUCKET = os.getenv("BUCKET_PREFIX", "local-grapl") + "-analyzers"
GRAPL_LOG_LEVEL = os.getenv("GRAPL_LOG_LEVEL", "ERROR")
LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(GRAPL_LOG_LEVEL)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

app = Chalice(app_name="analyzer-deployer")

ANALYZER_PARTITION = "analyzer"
ANALYZER_DEPLOYMENT_PARTITION = "analyzer_deployment"

DYNAMODB_CLIENT: dynamodb.DynamoDBServiceResource = (
    boto3.resource(
        "dynamodb",
        region_name="us-west-2",
        endpoint_url="http://dynamodb:8000",
        aws_access_key_id="dummy_cred_aws_access_key_id",
        aws_secret_access_key="dummy_cred_aws_secret_access_key",
    )
    if IS_LOCAL
    else boto3.resource("dynamodb")
)

#
# data model objects
#


@dataclasses.dataclass
class PortConfig:
    protocol: str = ""
    port: int = 0

    def into_json(self) -> Dict[str, Any]:
        return dataclasses.asdict(self)

    @classmethod
    def from_json(cls, data: Dict[str, Union[str, int]]) -> "PortConfig":
        return cls(**data)


@dataclasses.dataclass
class TableConfig:
    table: str = ""
    write: bool = False

    def into_json(self) -> Dict[str, Any]:
        return dataclasses.asdict(self)

    @classmethod
    def from_json(cls, data: Dict[str, Union[str, bool]]) -> "TableConfig":
        return cls(**data)


@dataclasses.dataclass
class SecretConfig:
    SecretId: str = ""
    VersionId: Optional[str] = None
    VersionStage: Optional[str] = None

    def into_json(self) -> Dict[str, Any]:
        return dataclasses.asdict(self)

    @classmethod
    def from_json(cls, data: Dict[str, str]) -> "SecretConfig":
        return cls(**data)


@dataclasses.dataclass
class AnalyzerConfig:
    requires_external_internet: Optional[Iterable[PortConfig]] = None
    requires_dynamodb: Optional[Iterable[TableConfig]] = None
    requires_graph: bool = False
    requires_secrets: Optional[Iterable[SecretConfig]] = None

    def into_json(self) -> Dict[str, Any]:
        return dataclasses.asdict(self)

    @classmethod
    def from_json(cls, data: Dict[str, Union[bool, Iterable[Any]]]) -> "AnalyzerConfig":
        return cls(
            requires_external_internet=(
                None
                if "requires_external_internet" not in data
                or data["requires_external_internet"] is None
                or len(data["requires_external_internet"]) == 0
                else [
                    PortConfig.from_json(d) for d in data["requires_external_internet"]
                ]
            ),
            requires_dynamodb=(
                None
                if "requires_dynamodb" not in data
                or data["requires_dynamodb"] is None
                or len(data["requires_dynamodb"]) == 0
                else [TableConfig.from_json(d) for d in data["requires_dynamodb"]]
            ),
            requires_graph=data["requires_graph"],
            requires_secrets=(
                None
                if "requires_secrets" not in data
                or data["requires_secrets"] is None
                or len(data["requires_secrets"]) == 0
                else [SecretConfig.from_json(d) for d in data["requires_secrets"]]
            ),
        )


@dataclasses.dataclass
class AnalyzerDeployment:
    analyzer_id: str = ""
    analyzer_version: int = 0
    s3_key: str = ""
    currently_deployed: bool = False
    last_deployed_time: Optional[int] = None
    analyzer_configuration: Optional[AnalyzerConfig] = None

    def into_json(self) -> Dict[str, Any]:
        return dataclasses.asdict(self)

    @classmethod
    def from_json(cls, data: Dict[str, Any]) -> "AnalyzerDeployment":
        return cls(
            analyzer_id=data["analyzer_id"],
            analyzer_version=data["analyzer_version"],
            s3_key=data["s3_key"],
            currently_deployed=data["currently_deployed"],
            last_deployed_time=data["last_deployed_time"],
            analyzer_configuration=(
                None
                if "analyzer_configuration" not in data
                or data["analyzer_configuration"] is None
                else AnalyzerConfig.from_json(data["analyzer_configuration"])
            ),
        )

    @classmethod
    def from_analyzer(cls, analyzer: "Analyzer") -> "AnalyzerDeployment":
        return cls(
            analyzer_id=analyzer.analyzer_id,
            analyzer_version=0,
            s3_key=_s3_key(analyzer_id=analyzer.analyzer_id, analyzer_version=0),
            currently_deployed=False,
        )


@dataclasses.dataclass
class Analyzer:
    analyzer_id: str = ""
    analyzer_active: bool = False
    created_time: int = 0
    last_update_time: int = 0

    def new_deployment(self) -> AnalyzerDeployment:
        return AnalyzerDeployment.from_analyzer(self)

    def into_json(self) -> Dict[str, Any]:
        return dataclasses.asdict(self)

    @classmethod
    def from_json(cls, data: Dict[str, Union[str, bool, int]]) -> "Analyzer":
        return cls(**data)

    @classmethod
    def create(cls) -> "Analyzer":
        now: Millis = as_millis(datetime.utcnow())
        return cls(
            analyzer_id=f"{uuid.uuid4()}",
            analyzer_active=True,
            created_time=now,
            last_update_time=now,
        )


@dataclasses.dataclass
class DynamoWrapper:
    partition_key: str = ""
    sort_key: str = ""
    analyzer_id: str = ""
    analyzer: Optional[Analyzer] = None
    analyzer_deployment: Optional[AnalyzerDeployment] = None

    def into_json(self) -> Dict[str, Any]:
        return dataclasses.asdict(self)

    def is_analyzer_row(self):
        return self.analyzer is not None

    @classmethod
    def from_json(cls, data: Dict[str, Union[str, Dict[str, Any]]]) -> "DynamoWrapper":
        analyzer = (
            None
            if "analyzer" not in data or data["analyzer"] is None
            else Analyzer.from_json(data["analyzer"])
        )
        analyzer_deployment = (
            None
            if "analyzer_deployment" not in data or data["analyzer_deployment"] is None
            else AnalyzerDeployment.from_json(data["analyzer_deployment"])
        )
        return cls(
            partition_key=data["partition_key"],
            sort_key=data["sort_key"],
            analyzer_id=data["analyzer_id"],
            analyzer=analyzer,
            analyzer_deployment=analyzer_deployment,
        )

    @classmethod
    def from_analyzer(cls, analyzer: Analyzer) -> "DynamoWrapper":
        return cls(
            partition_key=DynamoWrapper.partition_key(analyzer),
            sort_key=DynamoWrapper.sort_key(analyzer),
            analyzer=analyzer,
        )

    @classmethod
    def from_analyzer_deployment(
        cls, analyzer_deployment: AnalyzerDeployment
    ) -> "DynamoWrapper":
        return cls(
            partition_key=DynamoWrapper.partition_key(analyzer_deployment),
            sort_key=DynamoWrapper.sort_key(analyzer_deployment),
            analyzer_deployment=analyzer_deployment,
        )

    @staticmethod
    def partition_key(obj: Union[Analyzer, AnalyzerDeployment]) -> str:
        if isinstance(obj, Analyzer):
            return ANALYZER_PARTITION
        elif isinstance(obj, AnalyzerDeployment):
            return ANALYZER_DEPLOYMENT_PARTITION
        else:
            raise TypeError("Encountered object of unknown type")

    @staticmethod
    def sort_key(obj: Union[Analyzer, AnalyzerDeployment]) -> str:
        if isinstance(obj, Analyzer):
            return DynamoWrapper.analyzer_sort_key(obj.analyzer_id)
        elif isinstance(obj, AnalyzerDeployment):
            return DynamoWrapper.analyzer_deployment_sort_key(
                obj.analyzer_id, obj.analyzer_version
            )
        else:
            raise TypeError("Encountered object of unknown type")

    @staticmethod
    def analyzer_sort_key(analyzer_id: str) -> str:
        return f"{ANALYZER_PARTITION}-{analyzer_id}"

    @staticmethod
    def analyzer_deployment_sort_key(analyzer_id: str, analyzer_version: int) -> str:
        return f"{ANALYZER_DEPLOYMENT_PARTITION}-{analyzer_id}-{analyzer_version}"


@dataclasses.dataclass
class CreateAnalyzerResponse:
    analyzer_id: str = ""
    analyzer_version: int = 0
    s3_key: str = ""

    def into_json(self) -> Dict[str, Any]:
        return dataclasses.asdict(self)

    @classmethod
    def from_json(cls, data: Dict[str, Union[str, int]]) -> "CreateAnalyzerResponse":
        return cls(**data)

    @classmethod
    def from_analyzer_deployment(
        cls, analyzer_deployment: AnalyzerDeployment
    ) -> "CreateAnalyzerResponse":
        return CreateAnalyzerResponse(
            analyzer_id=analyzer_deployment.analyzer_id,
            analyzer_version=analyzer_deployment.analyzer_version,
            s3_key=analyzer_deployment.s3_key,
        )


T = TypeVar("T")


@dataclasses.dataclass
class PagingResponse(Generic[T]):
    limit: int = 0
    next_page: Optional[str] = None


@dataclasses.dataclass
class ListAnalyzersResponse(PagingResponse):
    analyzers: Iterable[Analyzer] = tuple()

    def into_json(self) -> Dict[str, Any]:
        return dataclasses.asdict(self)

    @classmethod
    def from_json(cls, data: Dict[str, Union[str, int]]):
        return cls(
            analyzers=[Analyzer.from_json(a) for a in data["analyzers"]],
            limit=data["limit"],
            next_page=data.get("next_page", None),
        )


@dataclasses.dataclass
class ListAnalyzerDeploymentsResponse(PagingResponse):
    analyzer_deployments: Iterable[AnalyzerDeployment] = tuple()

    def into_json(self) -> Dict[str, Any]:
        return dataclasses.asdict(self)

    @classmethod
    def from_json(cls, data: Dict[str, Union[str, int]]):
        return cls(
            analyzer_deployments=[
                AnalyzerDeployment.from_json(a) for a in data["analyzer_deployments"]
            ],
            limit=data["limit"],
            next_page=data.get("next_page", None),
        )


#
# helper functions
#


def _s3_key(analyzer_id: str, analyzer_version: int) -> str:
    return f"{ANALYZERS_BUCKET}/{analyzer_id}/{analyzer_version}/lambda.zip"


def _analyzers_table() -> dynamodb.ServiceResource.Table:
    return DYNAMODB_CLIENT.Table(ANALYZERS_TABLE)


#
# create analyzer
# POST /api/1/analyzers
#


def _create_analyzer(
    analyzers_table: dynamodb.ServiceResource.Table,
) -> CreateAnalyzerResponse:
    analyzer = Analyzer.create()
    analyzers_table.put_item(Item=DynamoWrapper.from_analyzer(analyzer).into_json())
    analyzer_deployment = analyzer.new_deployment()
    analyzers_table.put_item(
        Item=DynamoWrapper.from_analyzer_deployment(analyzer_deployment).into_json()
    )
    return CreateAnalyzerResponse.from_analyzer_deployment(analyzer_deployment)


@app.route("api/1/analyzers", methods=("POST",))
def create_analyzer() -> Dict[str, Any]:
    analyzers_table = _analyzers_table()
    return _create_analyzer(analyzers_table).into_json()


#
# list analyzers
# GET /api/1/analyzers?[active=(true|false|all)]&[limit=100]&[start]
#


def _extract_analyzers_paging_params() -> Tuple[
    Literal["true", "false", "all"], int, Optional[str]
]:
    active: Optional[Literal["true", "false", "all"]] = None
    limit: Optional[int] = None
    start: Optional[str] = None
    query_params = app.current_request.query_params
    if query_params is not None:
        active = query_params.get("active", "all")
        limit = int(query_params.get("limit", "100"))
        start = query_params.get("start", None)
    return active, limit, start


def _get_analyzers_page(
    analyzers_table: dynamodb.ServiceResource.Table,
    active: Literal["true", "false", "all"],
    limit: int,
    start: str = None,
) -> ListAnalyzersResponse:
    key_condition_expression = "partition_key = :partition_key_val AND begins_with (sort_key, :partition_key_val)"
    expression_attribute_values = {
        ":partition_key_val": ANALYZER_PARTITION,
    }
    filter_expression = (
        "analyzer.analyzer_active = :active_val"
        if active == "true" or active == "false"
        else None
    )
    query_kwargs = {
        "Select": "ALL_ATTRIBUTES",
        "KeyConditionExpression": key_condition_expression,
        "Limit": limit,
    }

    if start is not None:
        query_kwargs["ExclusiveStartKey"] = {
            "partition_key": ANALYZER_PARTITION,
            "sort_key": start,
        }

    if filter_expression is not None:
        expression_attribute_values[":active_val"] = active
        query_kwargs["FilterExpression"] = filter_expression

    query_kwargs["ExpressionAttributeValues"] = expression_attribute_values

    result: dynamodb.type_defs.QueryOutputTypeDef = analyzers_table.query(
        **query_kwargs
    )
    next_page = (
        result["LastEvaluatedKey"]["sort_key"] if "LastEvaluatedKey" in result else None
    )

    return ListAnalyzersResponse(
        limit,
        next_page,
        [DynamoWrapper.from_json(a).analyzer for a in result["Items"]],
    )


@app.route("api/1/analyzers", methods=("GET",))
def list_analyzers() -> Dict[str, Any]:
    analyzers_table = _analyzers_table()
    active, limit, start = _extract_analyzers_paging_params()
    return _get_analyzers_page(analyzers_table, active, limit, start).into_json()


#
# get analyzer
# GET /api/1/analyzers/<analyzer_id>
#


@app.route("api/1/analyzers/{analyzer_id}", methods=("GET",))
def get_analyzer(analyzer_id: str) -> Dict[str, Any]:
    analyzers_table = _analyzers_table()
    result = analyzers_table.get_item(
        Key={
            "partition_key": ANALYZER_PARTITION,
            "sort_key": DynamoWrapper.analyzer_sort_key(analyzer_id),
        }
    )
    if "Item" in result:
        dynamo_wrapper = DynamoWrapper.from_json(result["Item"])
        if dynamo_wrapper.is_analyzer_row():
            return dynamo_wrapper.analyzer.into_json()

    raise NotFoundError("Not found")


#
# deactivate analyzer
# DELETE /api/1/analyzers/<analyzer_id>
#


@app.route("api/1/analyzers/{analyzer_id}", methods=("DELETE",))
def deactivate_analyzer(analyzer_id: str):
    analyzers_table = _analyzers_table()
    analyzers_table.update_item()  # FIXME


#
# update analyzer
# POST /api/1/analyzers/<analyzer_id>/update
#


@app.route("api/1/analyzers/{analyzer_id}/update", methods=("POST",))
def update_analyzer(analyzer_id: str):
    analyzers_table = _analyzers_table()
    analyzers_table.update_item()  # FIXME


#
# deploy analyzer
# POST /api/1/analyzers/<analyzer_id>/deploy
#


def _create_queue(queue_name: str):
    client: sqs.Client = boto3.resource("sqs")
    client.create_queue()
    pass


@app.route("api/1/analyzers/{analyzer_id}/deploy", methods=("POST",))
def deploy_analyzer(analyzer_id: str):
    analyzers_table = _analyzers_table()
    analyzers_table.update_item()  # FIXME


#
# list analyzer deployments
# GET /api/1/analyzers/<analyzer_id>/deployments?[currently_deployed=(true|false|all)]&[limit=100]&[start]
#


def _extract_analyzer_deployments_paging_params() -> Tuple[
    Optional[Literal["true", "false", "all"]], Optional[int]
]:
    currently_deployed: Optional[bool] = None
    limit: Optional[int] = None
    query_params = app.current_request.query_params
    if query_params is not None:
        currently_deployed = query_params.get("currently_deployed", "all")
        limit = int(query_params.get("limit", "100"))
    return currently_deployed, limit


@app.route("api/1/analyzers/{analyzer_id}/deployments", methods=("GET",))
def list_analyzer_deployments(analyzer_id: str):
    analyzers_table = _analyzers_table()
    currently_deployed, limit = _extract_paging_params()
    # FIXME


#
# get analyzer deployment
# GET /api/1/analyzers/<analyzer_id>/deployments/<analyzer_version>
#


@app.route(
    "api/1/analyzers/{analyzer_id}/deployments/{analyzer_version}", methods=("GET",)
)
def get_analyzer_deployment(analyzer_id: str, analyzer_version: int):
    analyzers_table = _analyzers_table()
    result = analyzers_table.get_item(
        Key={
            "partition_key": ANALYZER_DEPLOYMENT_PARTITION,
            "sort_key": DynamoWrapper.analyzer_deployment_sort_key(
                analyzer_id, analyzer_version
            ),
        }
    )
    if "Item" in result:
        dynamo_wrapper = DynamoWrapper.from_json(result["Item"])
        if not dynamo_wrapper.is_analyzer_row():
            return dynamo_wrapper.analyzer_deployment.into_json()

    raise NotFoundError("Not found")


#
# teardown analyzer deployment
# DELETE /api/1/analyzers/<analyzer_id>/deployments/<analyzer_version>
#


@app.route(
    "api/1/analyzers/{analyzer_id}/deployments/{analyzer_version}", methods=("DELETE",)
)
def teardown_analyzer_deployment(analyzer_id: str, analyzer_version: int):
    analyzers_table = _analyzers_table()
    analyzers_table.update_item()  # FIXME
