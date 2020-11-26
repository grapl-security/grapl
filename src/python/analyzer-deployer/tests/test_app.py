import dataclasses
import functools
import json
import unittest
import uuid

import boto3
import botocore
import pytest

import hypothesis.strategies as st

from hypothesis import given
from chalice.test import Client, HTTPResponse
from mypy_boto3 import dynamodb

from analyzer_deployer.app import (
    ANALYZERS_BUCKET,
    ANALYZERS_TABLE,
    DYNAMODB_CLIENT,
    app,
    Analyzer,
    AnalyzerConfig,
    AnalyzerDeployment,
    DynamoWrapper,
    CreateAnalyzerResponse,
    ListAnalyzersResponse,
    ListAnalyzerDeploymentsResponse,
)
from chalice.test import Client
from hypothesis import given

UUID_REGEX = r"[a-f0-9]{8}\-[a-f0-9]{4}\-[a-f0-9]{4}\-[a-f0-9]{4}\-[a-f0-9]{12}"
KEY_SCHEMA = [
    {"KeyType": "HASH", "AttributeName": "partition_key"},
    {"KeyType": "RANGE", "AttributeName": "sort_key"},
]
ATTRIBUTE_DEFINITIONS = [
    {"AttributeName": "partition_key", "AttributeType": "S"},
    {"AttributeName": "sort_key", "AttributeType": "S"},
]


def _random_analyzers_table() -> str:
    return f"{ANALYZERS_TABLE}-{uuid.uuid4()}"


def _analyzers_table(table_name) -> dynamodb.ServiceResource.Table:
    table = None
    try:
        DYNAMODB_CLIENT.create_table(
            TableName=table_name,
            BillingMode="PAY_PER_REQUEST",
            AttributeDefinitions=ATTRIBUTE_DEFINITIONS,
            KeySchema=KEY_SCHEMA,
        )
        table = DYNAMODB_CLIENT.Table(table_name)
    except botocore.exceptions.ClientError:  # probably the table already exists
        table = DYNAMODB_CLIENT.Table(table_name)

    return table


# Monkeypatch the analyzer_deployer.app._analyzers_table() function to
# return a different test table instead. This keeps tests independent
# wrt DynamoDB. Make sure to decorate any tests which call this with
# @pytest.mark.forked
def _monkeypatch_analyzers_table(table_name):
    import analyzer_deployer.app

    analyzer_deployer.app._analyzers_table = functools.partial(
        _analyzers_table, table_name=table_name
    )


# Test that our schema hasn't drifted from the one defined in the
# etc/local_dynamo/provision_local_identity_table.py script. If this test
# fails, update the KEY_SCHEMA and ATTRIBUTE_DEFINITIONS above to
# match.
class TestAnalyzersTableSchemaDrift(unittest.TestCase):
    @pytest.mark.integration_test
    def test_analyzers_table_schema_drift(self):
        analyzers_table: dynamodb.ServiceResource.Table = DYNAMODB_CLIENT.Table(
            ANALYZERS_TABLE
        )
        self.assertListEqual(analyzers_table.key_schema, KEY_SCHEMA)
        self.assertListEqual(
            analyzers_table.attribute_definitions, ATTRIBUTE_DEFINITIONS
        )


#
# Hypothesis strategies
#


def _port_configs() -> st.SearchStrategy[PortConfig]:
    return st.builds(
        PortConfig,
        st.builds(
            dict,
            protocol=st.text(),
            port=st.integers(),
        ),
    )


def _table_configs():
    return st.builds(TableConfig, **{"table": st.text(), "write": st.booleans()})


def _secret_configs():
    return st.builds(
        SecretConfig,
        **{
            "SecretId": st.text(),
            "VersionId": st.text() | st.none(),
            "VersionStage": st.text() | st.none(),
        },
    )


def _analyzer_configs():
    return st.builds(
        AnalyzerConfig,
        **{
            "requires_external_internet": st.lists(_port_configs(), min_size=1)
            | st.none(),
            "requires_dynamodb": st.lists(_table_configs(), min_size=1) | st.none(),
            "requires_graph": st.booleans(),
            "requires_secrets": st.lists(_secret_configs(), min_size=1) | st.none(),
        },
    )


def _analyzer_deployments():
    return st.builds(
        AnalyzerDeployment,
        **{
            "analyzer_id": st.builds(str, st.uuids(version=4)),
            "analyzer_version": st.integers(),
            "s3_key": st.text(),
            "currently_deployed": st.booleans(),
            "last_deployed_time": st.integers() | st.none(),
            "analyzer_configuration": _analyzer_configs() | st.none(),
        },
    )


def _analyzers() -> st.SearchStrategy[Analyzer]:
    return st.builds(
        Analyzer,
        **{
            "analyzer_id": st.builds(str, st.uuids(version=4)),
            "analyzer_active": st.booleans(),
            "created_time": st.integers(),
            "last_update_time": st.integers(),
        },
    )


def _dynamo_wrappers() -> st.SearchStrategy[DynamoWrapper]:
    return st.builds(
        DynamoWrapper,
        **{
            "partition_key": st.text(),
            "sort_key": st.text(),
            "analyzer": _analyzers() | st.none(),
            "analyzer_deployment": _analyzer_deployments() | st.none(),
        },
    )


def _create_analyzer_responses():
    return st.builds(
        CreateAnalyzerResponse,
        **{
            "analyzer_id": st.builds(str, st.uuids(version=4)),
            "analyzer_version": st.integers(),
            "s3_key": st.text(),
        },
    )


def _list_analyzers_responses():
    return st.builds(
        ListAnalyzersResponse,
        **{
            "limit": st.integers(),
            "next_page": st.builds(str, st.uuids(version=4)) | st.none(),
            "analyzers": st.lists(_analyzers()),
        },
    )


def _list_analyzer_deployments_responses():
    return st.builds(
        ListAnalyzerDeploymentsResponse,
        **{
            "limit": st.integers(),
            "next_page": st.builds(str, st.uuids(version=4)) | st.none(),
            "analyzer_deployments": st.lists(_analyzer_deployments()),
        },
    )


#
# Serde tests for data model objects
#


class TestSerde(unittest.TestCase):
    @given(_analyzers())
    def test_analyzer_encode_decode_invariant(self, analyzer: Analyzer):
        serialized = json.dumps(dataclasses.asdict(analyzer))
        deserialized = json.loads(serialized)
        self.assertDictEqual(deserialized, analyzer.into_json())

    @given(_analyzers())
    def test_analyzer_from_json(self, analyzer: Analyzer):
        self.assertEqual(Analyzer.from_json(analyzer.into_json()), analyzer)

    @given(_analyzer_deployments())
    def test_analyzer_deployment_encode_decode_invariant(
        self, analyzer_deployment: AnalyzerDeployment
    ):
        serialized = json.dumps(dataclasses.asdict(analyzer_deployment))
        deserialized = json.loads(serialized)
        self.assertDictEqual(deserialized, analyzer_deployment.into_json())

    @given(_analyzer_deployments())
    def test_analyzer_deployment_from_json(
        self, analyzer_deployment: AnalyzerDeployment
    ):
        self.assertEqual(
            AnalyzerDeployment.from_json(analyzer_deployment.into_json()),
            analyzer_deployment,
        )

    @given(_dynamo_wrappers())
    def test_dynamo_wrapper_encode_decode_invariant(
        self, dynamo_wrapper: DynamoWrapper
    ):
        serialized = json.dumps(dataclasses.asdict(dynamo_wrapper))
        deserialized = json.loads(serialized)
        self.assertDictEqual(deserialized, dynamo_wrapper.into_json())

    @given(_dynamo_wrappers())
    def test_dynamo_wrapper_from_json(self, dynamo_wrapper: DynamoWrapper):
        self.assertEqual(
            DynamoWrapper.from_json(dynamo_wrapper.into_json()), dynamo_wrapper
        )

    @given(_create_analyzer_responses())
    def test_create_analyzer_response_encode_decode_invariant(
        self, create_analyzer_response: CreateAnalyzerResponse
    ):
        serialized = json.dumps(dataclasses.asdict(create_analyzer_response))
        deserialized = json.loads(serialized)
        self.assertDictEqual(deserialized, create_analyzer_response.into_json())

    @given(_create_analyzer_responses())
    def test_create_analyzer_response_from_json(
        self, create_analyzer_response: CreateAnalyzerResponse
    ):
        self.assertEqual(
            CreateAnalyzerResponse.from_json(create_analyzer_response.into_json()),
            create_analyzer_response,
        )

    @given(_list_analyzers_responses())
    def test_list_analyzers_response_encode_decode_invariant(
        self, list_analyzers_response: ListAnalyzersResponse
    ):
        serialized = json.dumps(dataclasses.asdict(list_analyzers_response))
        deserialized = json.loads(serialized)
        self.assertDictEqual(deserialized, list_analyzers_response.into_json())

    @given(_list_analyzers_responses())
    def test_list_analyzers_response_from_json(
        self, list_analyzers_response: ListAnalyzersResponse
    ):
        self.assertEqual(
            ListAnalyzersResponse.from_json(list_analyzers_response.into_json()),
            list_analyzers_response,
        )

    @given(_list_analyzer_deployments_responses())
    def test_list_analyzer_deployments_responses_encode_decode_invariant(
        self, list_analyzer_deployments_response: ListAnalyzerDeploymentsResponse
    ):
        serialized = json.dumps(dataclasses.asdict(list_analyzer_deployments_response))
        deserialized = json.loads(serialized)
        self.assertDictEqual(
            deserialized, list_analyzer_deployments_response.into_json()
        )

    @given(_list_analyzer_deployments_responses())
    def test_list_analyzer_deployments_response_from_json(
        self, list_analyzer_deployments_response: ListAnalyzerDeploymentsResponse
    ):
        self.assertEqual(
            ListAnalyzerDeploymentsResponse.from_json(
                list_analyzer_deployments_response.into_json()
            ),
            list_analyzer_deployments_response,
        )


#
# Application integration tests against DynamoDB
#


class TestApp(unittest.TestCase):
    @pytest.mark.forked
    @pytest.mark.integration_test
    def test_create_analyzer(self):
        with Client(app) as client:
            _monkeypatch_analyzers_table(table_name=_random_analyzers_table())

            create_response: HTTPResponse = client.http.post("api/1/analyzers")
            self.assertEqual(create_response.status_code, 200)
            create_analyzer_response = CreateAnalyzerResponse.from_json(
                create_response.json_body
            )
            self.assertRegex(
                create_analyzer_response.analyzer_id,
                UUID_REGEX,
            )
            self.assertEqual(create_analyzer_response.analyzer_version, 0)
            self.assertEqual(
                create_analyzer_response.s3_key,
                f"{ANALYZERS_BUCKET}/{create_analyzer_response.analyzer_id}/{create_analyzer_response.analyzer_version}/lambda.zip",
            )

            get_analyzer_response: HTTPResponse = client.http.get(
                f"api/1/analyzers/{create_analyzer_response.analyzer_id}"
            )
            self.assertEqual(get_analyzer_response.status_code, 200)
            analyzer = Analyzer.from_json(get_analyzer_response.json_body)
            self.assertEqual(analyzer.analyzer_id, create_analyzer_response.analyzer_id)
            self.assertTrue(analyzer.analyzer_active)
            self.assertGreater(analyzer.created_time, 0)
            self.assertEqual(analyzer.last_update_time, analyzer.created_time)

            get_analyzer_deployment_response: HTTPResponse = client.http.get(
                f"api/1/analyzers/{create_analyzer_response.analyzer_id}/deployments/{create_analyzer_response.analyzer_version}"
            )
            self.assertEqual(get_analyzer_deployment_response.status_code, 200)
            analyzer_deployment = AnalyzerDeployment.from_json(
                get_analyzer_deployment_response.json_body
            )
            self.assertEqual(
                analyzer_deployment.analyzer_id, create_analyzer_response.analyzer_id
            )
            self.assertFalse(analyzer_deployment.currently_deployed)
            self.assertIsNone(analyzer_deployment.last_deployed_time)
            self.assertIsNone(analyzer_deployment.analyzer_configuration)

    @pytest.mark.forked
    @pytest.mark.integration_test
    def test_list_analyzers(self):
        _monkeypatch_analyzers_table(table_name=_random_analyzers_table())

        with Client(app) as client:
            create_response_1: HTTPResponse = client.http.post("api/1/analyzers")
            self.assertEqual(create_response_1.status_code, 200)
            create_analyzer_response_1 = CreateAnalyzerResponse.from_json(
                create_response_1.json_body
            )

            create_response_2: HTTPResponse = client.http.post("api/1/analyzers")
            self.assertEqual(create_response_2.status_code, 200)
            create_analyzer_response_2 = CreateAnalyzerResponse.from_json(
                create_response_2.json_body
            )

            create_response_3: HTTPResponse = client.http.post("api/1/analyzers")
            self.assertEqual(create_response_3.status_code, 200)
            create_analyzer_response_3 = CreateAnalyzerResponse.from_json(
                create_response_3.json_body
            )

            analyzer_ids = sorted(
                i
                for i in [
                    create_analyzer_response_1.analyzer_id,
                    create_analyzer_response_2.analyzer_id,
                    create_analyzer_response_3.analyzer_id,
                ]
            )

            list_response_1: HTTPResponse = client.http.get("api/1/analyzers?limit=2")
            self.assertEqual(list_response_1.status_code, 200)
            list_analyzers_response_1 = ListAnalyzersResponse.from_json(
                list_response_1.json_body
            )
            self.assertEqual(list_analyzers_response_1.limit, 2)
            self.assertEqual(len(list_analyzers_response_1.analyzers), 2)
            self.assertIsNotNone(list_analyzers_response_1.next_page)
            self.assertListEqual(
                [a.analyzer_id for a in list_analyzers_response_1.analyzers],
                analyzer_ids[0:2],
            )

            list_response_2: HTTPResponse = client.http.get(
                f"api/1/analyzers?limit=2&start={list_analyzers_response_1.next_page}"
            )
            self.assertEqual(list_response_2.status_code, 200)
            list_analyzers_response_2 = ListAnalyzersResponse.from_json(
                list_response_2.json_body
            )
            self.assertEqual(list_analyzers_response_2.limit, 2)
            self.assertEqual(len(list_analyzers_response_2.analyzers), 1)
            self.assertIsNone(list_analyzers_response_2.next_page)
            self.assertListEqual(
                [a.analyzer_id for a in list_analyzers_response_2.analyzers],
                analyzer_ids[2:],
            )

    @pytest.mark.forked
    @pytest.mark.integration_test
    def test_list_analyzer_deployments(self):
        _monkeypatch_analyzers_table(table_name=_random_analyzers_table())

        with Client(app) as client:
            create_response_1: HTTPResponse = client.http.post("api/1/analyzers")
            self.assertEqual(create_response_1.status_code, 200)
            create_analyzer_response_1 = CreateAnalyzerResponse.from_json(
                create_response_1.json_body
            )

            deploy_response_1: HTTPResponse = client.http.post(
                f"api/1/analyzers/{create_analyzer_response_1.analyzer_id}/deploy"
            )
            deploy_response_2: HTTPResponse = client.http.post(
                f"api/1/analyzers/{create_analyzer_response_1.analyzer_id}/deploy"
            )
            deploy_response_3: HTTPResponse = client.http.post(
                f"api/1/analyzers/{create_analyzer_response_1.analyzer_id}/deploy"
            )

            analyzer_versions = sorted(
                i
                for i in [
                    deploy_analyzer_response_1.analyzer_version,
                    deploy_analyzer_response_2.analyzer_version,
                    deploy_analyzer_response_3.analyzer_version,
                ]
            )

            list_response_1: HTTPResponse = client.http.get("api/1/analyzers?limit=2")
            self.assertEqual(list_response_1.status_code, 200)
            list_analyzer_deployments_response_1 = (
                ListAnalyzerDeploymentsResponse.from_json(list_response_1.json_body)
            )
            self.assertEqual(list_analyzer_deployments_response_1.limit, 2)
            self.assertEqual(len(list_analyzer_deployments_response_1.analyzers), 2)
            self.assertIsNotNone(list_analyzer_deployments_response_1.next_page)
            self.assertListEqual(
                [
                    d.analyzer_version
                    for d in list_analyzer_deployments_response_1.analyzer_deployments
                ],
                analyzer_versions[0:2],
            )

            list_response_2: HTTPResponse = client.http.get("api/1/analyzers?limit=2")
            self.assertEqual(list_response_2.status_code, 200)
            list_analyzer_deployments_response_2 = (
                ListAnalyzerDeploymentsResponse.from_json(list_response_2.json_body)
            )
            self.assertEqual(list_analyzer_deployments_response_2.limit, 2)
            self.assertEqual(len(list_analyzer_deployments_response_2.analyzers), 1)
            self.assertIsNone(list_analyzer_deployments_response_2.next_page)
            self.assertListEqual(
                [
                    d.analyzer_version
                    for d in list_analyzer_deployments_response_2.analyzer_deployments
                ],
                analyzer_versions[0:2],
            )

    @pytest.mark.forked
    @pytest.mark.integration_test
    def test_deactivate_analyzers(self):
        with Client(app) as client:
            _monkeypatch_analyzers_table(table_name=_random_analyzers_table())

            create_response: HTTPResponse = client.http.post("api/1/analyzers")
            self.assertEqual(create_response.status_code, 200)
            create_analyzer_response = CreateAnalyzerResponse.from_json(
                create_response.json_body
            )

            get_response_1: HTTPResponse = client.http.get(
                f"api/1/analyzers/{create_analyzer_response.analyzer_id}"
            )
            self.assertEqual(get_response_1.status_code, 200)
            get_analyzer_response_1 = Analyzer.from_json(get_response_1.json_body)
            self.assertEqual(
                get_analyzer_response_1.analyzer_id,
                create_analyzer_response.analyzer_id,
            )
            self.assertTrue(get_analyzer_response_1.analyzer_active)

            deactivate_response = client.http.delete(
                f"api/1/analyzers/{create_analyzer_response.analyzer_id}"
            )
            self.assertEqual(deactivate_response.status_code, 202)

            get_response_2: HTTPResponse = client.http.get(
                f"api/1/analyzers/{create_analyzer_response.analyzer_id}"
            )
            self.assertEqual(get_response_2.status_code, 200)
            get_analyzer_response_2 = Analyzer.from_json(get_response_2.json_body)
            self.assertEqual(
                get_analyzer_response_2.analyzer_id,
                create_analyzer_response.analyzer_id,
            )
            self.assertFalse(get_analyzer_response_2.analyzer_active)

    @pytest.mark.forked
    @pytest.mark.integration_test
    def test_deactivate_analyzers_returns_404_when_analyzer_does_not_exist(self):
        with Client(app) as client:
            _monkeypatch_analyzers_table(table_name=_random_analyzers_table())

            deactivate_response = client.http.delete(f"api/1/analyzers/{uuid.uuid4()}")
            self.assertEqual(deactivate_response.status_code, 404)

    @pytest.mark.forked
    @pytest.mark.integration_test
    def test_deactivate_analyzers_returns_409_when_analyzer_currently_deployed(self):
        with Client(app) as client:
            _monkeypatch_analyzers_table(table_name=_random_analyzers_table())

            create_response: HTTPResponse = client.http.post("api/1/analyzers")
            self.assertEqual(create_response.status_code, 200)
            create_analyzer_response = CreateAnalyzerResponse.from_json(
                create_response.json_body
            )

            get_response_1: HTTPResponse = client.http.get(
                f"api/1/analyzers/{create_analyzer_response.analyzer_id}"
            )
            self.assertEqual(get_response_1.status_code, 200)
            get_analyzer_response_1 = Analyzer.from_json(get_response_1.json_body)
            self.assertEqual(
                get_analyzer_response_1.analyzer_id,
                create_analyzer_response.analyzer_id,
            )
            self.assertTrue(get_analyzer_response_1.analyzer_active)

            deactivate_response = client.http.delete(
                f"api/1/analyzers/{create_analyzer_response.analyzer_id}"
            )
            self.assertEqual(deactivate_response.status_code, 202)

            get_response_2: HTTPResponse = client.http.get(
                f"api/1/analyzers/{create_analyzer_response.analyzer_id}"
            )
            self.assertEqual(get_response_2.status_code, 200)
            get_analyzer_response_2 = Analyzer.from_json(get_response_2.json_body)
            self.assertEqual(
                get_analyzer_response_2.analyzer_id,
                create_analyzer_response.analyzer_id,
            )
            self.assertFalse(get_analyzer_response_2.analyzer_active)
