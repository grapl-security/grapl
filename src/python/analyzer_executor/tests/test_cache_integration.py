import unittest
from redis import Redis

import pytest
from analyzer_executor_lib.analyzer_executor import prelude, check_msg_cache, update_msg_cache, check_hit_cache, update_hit_cache

SAMPLE_HOST = "localhost"
SAMPLE_PORT = "12345"

@pytest.fixture
def mock_cache_env(monkeypatch):
    def _mock_cache_env(addr, port):
        monkeypatch.setenv("MESSAGECACHE_ADDR", addr)
        monkeypatch.setenv("MESSAGECACHE_PORT", port)
        monkeypatch.setenv("HITCACHE_ADDR", addr)
        monkeypatch.setenv("HITCACHE_PORT", port)
    return _mock_cache_env

@pytest.mark.integration_test
def test_hit_cache_noop(mock_cache_env) -> None:
    mock_cache_env("", "")
    prelude()

    assert not check_hit_cache("a", "b")
    update_hit_cache("a", "b")
    assert not check_hit_cache("a", "b")

    mock_cache_env(SAMPLE_HOST, "")
    prelude()

    assert not check_hit_cache("a", "b")
    update_hit_cache("a", "b")
    assert not check_hit_cache("a", "b")

    mock_cache_env("", SAMPLE_PORT)
    prelude()

    assert not check_hit_cache("a", "b")
    update_hit_cache("a", "b")
    assert not check_hit_cache("a", "b")

@pytest.mark.integration_test
def test_hit_cache_redis(mock_cache_env) -> None:
    # no mock, environment should be configured to pick up Redis
    prelude()

    assert not check_hit_cache("a", "b")
    update_hit_cache("a", "b")
    assert check_hit_cache("a", "b")

@pytest.mark.integration_test
def test_message_cache_noop(mock_cache_env) -> None:
    mock_cache_env("", "")
    prelude()

    assert not check_msg_cache("a", "b", "c")
    update_msg_cache("a", "b", "c")
    assert not check_msg_cache("a", "b", "c")

    mock_cache_env(SAMPLE_HOST, "")
    prelude()

    assert not check_msg_cache("a", "b", "c")
    update_msg_cache("a", "b", "c")
    assert not check_msg_cache("a", "b", "c")

    mock_cache_env("", SAMPLE_PORT)
    prelude()

    assert not check_msg_cache("a", "b", "c")
    update_msg_cache("a", "b", "c")
    assert not check_msg_cache("a", "b", "c")

@pytest.mark.integration_test
def test_message_cache_redis(mock_cache_env) -> None:
    # no mock, environment should be configured to pick up Redis
    prelude()

    assert not check_msg_cache("a", "b", "c")
    update_msg_cache("a", "b", "c")
    assert check_msg_cache("a", "b", "c")
