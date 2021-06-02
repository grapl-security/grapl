import dataclasses
import json
import unittest

import hypothesis.strategies as st
import pytest
from analyzer_deployer.app import (
    Analyzer,
    AnalyzerConfig,
    AnalyzerDeployment,
    CreateAnalyzerResponse,
    PortConfig,
    SecretConfig,
    TableConfig,
    app,
)
from chalice.test import Client
from hypothesis import given


def _analyzers() -> st.SearchStrategy[Analyzer]:
    return st.builds(
        Analyzer,
        st.builds(
            dict,
            analyzer_id=st.text(min_size=1, max_size=64),
            analyzer_versions=st.lists(st.integers()),
            analyzer_active=st.booleans(),
            created_time=st.integers(),
            last_update_time=st.integers(),
        ),
    )


def _port_configs() -> st.SearchStrategy[PortConfig]:
    return st.builds(
        PortConfig,
        st.builds(
            dict,
            protocol=st.text(min_size=1, max_size=64),
            port=st.integers(),
        ),
    )


def _table_configs() -> st.SearchStrategy[TableConfig]:
    return st.builds(
        TableConfig,
        st.builds(dict, table=st.text(min_size=1, max_size=64), write=st.booleans()),
    )


def _secret_configs() -> st.SearchStrategy[SecretConfig]:
    return st.builds(
        SecretConfig,
        st.builds(
            dict,
            SecretId=st.text(min_size=1, max_size=64),
            VersionId=st.text(min_size=1, max_size=64),
            VersionStage=st.text(min_size=1, max_size=64),
        ),
    )


def _analyzer_configs() -> st.SearchStrategy[AnalyzerConfig]:
    return st.builds(
        AnalyzerConfig,
        st.builds(
            dict,
            requires_external_internet=st.lists(_port_configs()),
            requires_dynamodb=st.lists(_table_configs()),
            requires_graph=st.booleans(),
            requires_secrets=st.lists(_secret_configs()),
        ),
    )


def _analyzer_deployments() -> st.SearchStrategy[AnalyzerDeployment]:
    return st.builds(
        AnalyzerDeployment,
        st.builds(
            dict,
            analyzer_id=st.text(min_size=1, max_size=64),
            analyzer_version=st.integers(),
            s3_key=st.text(min_size=1, max_size=64),
            currently_deployed=st.booleans(),
            last_deployed_time=st.integers(),
            analyzer_configuration=_analyzer_configs(),
        ),
    )


def _create_analyzer_responses() -> st.SearchStrategy[CreateAnalyzerResponse]:
    return st.builds(
        CreateAnalyzerResponse,
        st.builds(
            dict,
            analyzer_id=st.text(min_size=1, max_size=64),
            analyzer_version=st.integers(),
            s3_key=st.text(min_size=1, max_size=64),
        ),
    )


class TestSerde(unittest.TestCase):
    @given(_analyzers())
    def test_analyzer_encode_decode_invariant(self, analyzer: Analyzer) -> None:
        serialized = json.dumps(dataclasses.asdict(analyzer))
        deserialized = json.loads(serialized)
        self.assertDictEqual(deserialized, dataclasses.asdict(analyzer))

    @given(_analyzer_deployments())
    def test_analyzer_deployment_encode_decode_invariant(
        self, analyzer_deployment: AnalyzerDeployment
    ) -> None:
        serialized = json.dumps(dataclasses.asdict(analyzer_deployment))
        deserialized = json.loads(serialized)
        self.assertDictEqual(deserialized, dataclasses.asdict(analyzer_deployment))

    @given(_create_analyzer_responses())
    def test_create_analyzer_response_encode_decode_invariant(
        self, create_analyzer_response: CreateAnalyzerResponse
    ) -> None:
        serialized = json.dumps(dataclasses.asdict(create_analyzer_response))
        deserialized = json.loads(serialized)
        self.assertDictEqual(deserialized, dataclasses.asdict(create_analyzer_response))


class TestApp(unittest.TestCase):
    def test_create_analyzer(self) -> None:
        with Client(app) as client:
            response = client.http.post("1/analyzers")
            self.assertIsNotNone(response.body)

    @pytest.mark.integration_test
    def test_create_analyzer_integration(self) -> None:
        pass
