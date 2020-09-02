import dataclasses
import json
import unittest

import pytest

import hypothesis.strategies as st

from hypothesis import given
from chalice.test import Client

from analyzer_deployer.app import (
    app,
    Analyzer,
    PortConfig,
    SecretConfig,
    TableConfig,
    AnalyzerConfig,
    AnalyzerDeployment,
    CreateAnalyzerResponse,
)


def _analyzers() -> st.SearchStrategy[Analyzer]:
    return st.builds(
        Analyzer,
        st.builds(
            dict,
            analyzer_id=st.text(),
            analyzer_versions=st.lists(st.integers()),
            analyzer_active=st.booleans(),
            created_time=st.integers(),
            last_update_time=st.integers(),
        ),
    )


def _port_configs() -> st.SearchStrategy[PortConfig]:
    return st.builds(
        PortConfig, st.builds(dict, protocol=st.text(), port=st.integers(),),
    )


def _table_configs():
    return st.builds(TableConfig, **{"table": st.text(), "write": st.booleans()})


def _secret_configs():
    return st.builds(
        SecretConfig,
        **{"SecretId": st.text(), "VersionId": st.text(), "VersionStage": st.text()}
    )


def _analyzer_configs():
    return st.builds(
        AnalyzerConfig,
        **{
            "requires_external_internet": st.lists(_port_configs()),
            "requires_dynamodb": st.lists(_table_configs()),
            "requires_graph": st.booleans(),
            "requires_secrets": st.lists(_secret_configs()),
        }
    )


def _analyzer_deployments():
    return st.builds(
        AnalyzerDeployment,
        **{
            "analyzer_id": st.text(),
            "analyzer_version": st.integers(),
            "s3_key": st.text(),
            "currently_deployed": st.booleans(),
            "last_deployed_time": st.integers(),
            "analyzer_configuration": _analyzer_configs(),
        }
    )


def _create_analyzer_responses():
    return st.builds(
        CreateAnalyzerResponse,
        **{
            "analyzer_id": st.text(),
            "analyzer_version": st.integers(),
            "s3_key": st.text(),
        }
    )


class TestSerde(unittest.TestCase):
    @given(_analyzers())
    def test_analyzer_encode_decode_invariant(self, analyzer: Analyzer):
        serialized = json.dumps(dataclasses.asdict(analyzer))
        deserialized = json.loads(serialized)
        self.assertDictEqual(deserialized, dataclasses.asdict(analyzer))

    @given(_analyzer_deployments())
    def test_analyzer_deployment_encode_decode_invariant(self, analyzer_deployment):
        serialized = json.dumps(dataclasses.asdict(analyzer_deployment))
        deserialized = json.loads(serialized)
        self.assertDictEqual(deserialized, dataclasses.asdict(analyzer_deployment))

    @given(_create_analyzer_responses())
    def test_create_analyzer_response_encode_decode_invariant(
        self, create_analyzer_response
    ):
        serialized = json.dumps(dataclasses.asdict(create_analyzer_response))
        deserialized = json.loads(serialized)
        self.assertDictEqual(deserialized, dataclasses.asdict(create_analyzer_response))


class TestApp(unittest.TestCase):
    def test_create_analyzer(self):
        with Client(app) as client:
            response = client.http.post("1/analyzers")
            self.assertIsNotNone(response.body)

    @pytest.mark.integration_test
    def test_create_analyzer_integration(self):
        pass
