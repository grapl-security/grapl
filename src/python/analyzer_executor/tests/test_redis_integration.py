import unittest

import pytest
from analyzer_executor_lib.analyzer_executor import NopCache, hit_cache, message_cache


class TestRedisIntegration(unittest.TestCase):
    @pytest.mark.integration_test
    def test_call_something_from_analyzer_executor_lib(self) -> None:
        assert isinstance(hit_cache, NopCache)
