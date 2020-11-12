import unittest
from redis import Redis

import pytest
from analyzer_executor_lib.analyzer_executor import AnalyzerExecutor

SAMPLE_ADDR = "localhost"
SAMPLE_PORT = "12345"

@pytest.fixture
def AnalyzerExecutorSingleton(monkeypatch):
    def _AnalyzerExecutorSingleton(stub_env=False, env_addr="", env_port=""):
        with monkeypatch.context() as mp:
            if stub_env:
                mp.setenv("MESSAGECACHE_ADDR", env_addr)
                mp.setenv("HITCACHE_ADDR",     env_addr)
                mp.setenv("MESSAGECACHE_PORT", env_port)
                mp.setenv("HITCACHE_PORT",     env_port)

            # force singleton to reinitialize,
            # this should be idempotent?
            AnalyzerExecutor._singleton = None
            return AnalyzerExecutor.singleton()
    return _AnalyzerExecutorSingleton

@pytest.mark.integration_test
def test_hit_cache_noop(AnalyzerExecutorSingleton) -> None:
    """
    Initializes the AnalyzerExecutor singleton without valid environment
    variables for a Redis cache connection, expecting hit cache hits to miss.
    """
    ae = AnalyzerExecutorSingleton(stub_env=True, env_addr="", env_port="")

    assert not ae.check_hit_cache("a", "b")
    ae.update_hit_cache("a", "b")
    assert not ae.check_hit_cache("a", "b")

    ae = AnalyzerExecutorSingleton(stub_env=True, env_addr=SAMPLE_ADDR, env_port="")

    assert not ae.check_hit_cache("a", "b")
    ae.update_hit_cache("a", "b")
    assert not ae.check_hit_cache("a", "b")

    ae = AnalyzerExecutorSingleton(stub_env=True, env_addr="", env_port=SAMPLE_PORT)

    assert not ae.check_hit_cache("a", "b")
    ae.update_hit_cache("a", "b")
    assert not ae.check_hit_cache("a", "b")

@pytest.mark.integration_test
def test_hit_cache_redis(AnalyzerExecutorSingleton) -> None:
    # defaults cribbed from actual build environment
    ae = AnalyzerExecutorSingleton(stub_env=False)

    assert not ae.check_hit_cache("a", "b")
    ae.update_hit_cache("a", "b")
    assert ae.check_hit_cache("a", "b")

@pytest.mark.integration_test
def test_message_cache_noop(AnalyzerExecutorSingleton) -> None:
    """
    Initializes the AnalyzerExecutor singleton without valid environment
    variables for a Redis cache connection, expecting hit cache hits to miss.
    """
    ae = AnalyzerExecutorSingleton(stub_env=True, env_addr="", env_port="")

    assert not ae.check_msg_cache("a", "b", "c")
    ae.update_msg_cache("a", "b", "c")
    assert not ae.check_msg_cache("a", "b", "c")

    ae = AnalyzerExecutorSingleton(stub_env=True, env_addr=SAMPLE_ADDR, env_port="")

    assert not ae.check_msg_cache("a", "b", "c")
    ae.update_msg_cache("a", "b", "c")
    assert not ae.check_msg_cache("a", "b", "c")

    ae = AnalyzerExecutorSingleton(stub_env=True, env_addr="", env_port=SAMPLE_PORT)

    assert not ae.check_msg_cache("a", "b", "c")
    ae.update_msg_cache("a", "b", "c")
    assert not ae.check_msg_cache("a", "b", "c")

@pytest.mark.integration_test
def test_message_cache_redis(AnalyzerExecutorSingleton) -> None:
    # defaults cribbed from actual build environment
    ae = AnalyzerExecutorSingleton(stub_env=False)

    assert not ae.check_msg_cache("a", "b", "c")
    ae.update_msg_cache("a", "b", "c")
    assert ae.check_msg_cache("a", "b", "c")
