import dataclasses
import json
import unittest

import hypothesis.strategies as st

from hypothesis import given
from chalice.test import Client

from analyzer_deployer.app import (
    app, Analyzer, PortConfig, SecretConfig, TableConfig, AnalyzerConfig,
    AnalyzerDeployment, CreateAnalyzerResponse
)


class TestAnalyzer(unittest.TestCase):
    @given(
        st.text(), st.lists(st.integers()), st.booleans(), st.integers(), st.integers()
    )
    def test_analyzer_encode_decode_invariant(
        self,
        analyzer_id,
        analyzer_versions,
        analyzer_active,
        created_time,
        last_update_time,
    ):
        analyzer = Analyzer(
            analyzer_id,
            analyzer_versions,
            analyzer_active,
            created_time,
            last_update_time,
        )
        serialized = json.dumps(dataclasses.asdict(analyzer))
        deserialized = json.loads(serialized)
        self.assertDictEqual(deserialized, dataclasses.asdict(analyzer))

    def test_


class TestApp(unittest.TestCase):
    def test_create_analyzer(self):
        with Client(app) as client:
            response = client.http.post("1/analyzers")
            self.assertIsNotNone(response.body)
