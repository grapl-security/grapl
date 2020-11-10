import unittest

import pytest

# the divergent `from src.` here - compared with in `lambda_function.py` - annoys me, and could be solved
# with some PATH hacking, but... that's also gross. Eh. Whatever.
from src.analyzer_executor_lib.analyzer_executor import (
    NopCache,
    hit_cache,
    message_cache,
)


class TestRedisIntegration(unittest.TestCase):
    @pytest.mark.integration_test
    def test_call_something_from_analyzer_executor_lib(self) -> None:
        assert isinstance(hit_cache, NopCache)
