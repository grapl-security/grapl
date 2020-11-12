import unittest
import pytest
from faker import Faker

from analyzer_executor_lib.analyzer_executor import AnalyzerExecutor

SAMPLE_ADDR = "localhost"
SAMPLE_PORT = "12345"


@pytest.fixture
def random_word():
    fake = Faker()

    def _random_word():
        return "-".join(fake.words(nb=4))

    return _random_word


@pytest.fixture
def AnalyzerExecutorSingleton(monkeypatch):
    def _AnalyzerExecutorSingleton(stub_env=False, env_addr="", env_port=""):
        with monkeypatch.context() as mp:
            if stub_env:
                mp.setenv("MESSAGECACHE_ADDR", env_addr)
                mp.setenv("HITCACHE_ADDR", env_addr)
                mp.setenv("MESSAGECACHE_PORT", env_port)
                mp.setenv("HITCACHE_PORT", env_port)

            # force singleton to reinitialize,
            # this should be idempotent?
            AnalyzerExecutor._singleton = None
            return AnalyzerExecutor.singleton()

    return _AnalyzerExecutorSingleton


@pytest.mark.integration_test
def test_hit_cache_noop(AnalyzerExecutorSingleton, random_word) -> None:
    """
    Initializes the AnalyzerExecutor singleton without valid environment
    variables for a Redis cache connection, expecting hit cache hits to miss.
    """
    ae = AnalyzerExecutorSingleton(stub_env=True, env_addr="", env_port="")

    k1, k2 = random_word(), random_word()

    assert not ae.check_hit_cache(k1, k2)
    ae.update_hit_cache(k1, k2)
    assert not ae.check_hit_cache(k1, k2)

    ae = AnalyzerExecutorSingleton(stub_env=True, env_addr=SAMPLE_ADDR, env_port="")

    assert not ae.check_hit_cache(k1, k2)
    ae.update_hit_cache(k1, k2)
    assert not ae.check_hit_cache(k1, k2)

    ae = AnalyzerExecutorSingleton(stub_env=True, env_addr="", env_port=SAMPLE_PORT)

    assert not ae.check_hit_cache(k1, k2)
    ae.update_hit_cache(k1, k2)
    assert not ae.check_hit_cache(k1, k2)


@pytest.mark.integration_test
def test_hit_cache_redis(AnalyzerExecutorSingleton, random_word) -> None:
    ae = AnalyzerExecutorSingleton(stub_env=False)

    k1, k2 = random_word(), random_word()

    assert not ae.check_hit_cache(k1, k2)
    ae.update_hit_cache(k1, k2)
    assert ae.check_hit_cache(k1, k2)


@pytest.mark.integration_test
def test_message_cache_noop(AnalyzerExecutorSingleton, random_word) -> None:
    """
    Initializes the AnalyzerExecutor singleton without valid environment
    variables for a Redis cache connection, expecting hit cache hits to miss.
    """
    ae = AnalyzerExecutorSingleton(stub_env=True, env_addr="", env_port="")

    k1, k2, k3 = random_word(), random_word(), random_word()

    assert not ae.check_msg_cache(k1, k2, k3)
    ae.update_msg_cache(k1, k2, k3)
    assert not ae.check_msg_cache(k1, k2, k3)

    ae = AnalyzerExecutorSingleton(stub_env=True, env_addr=SAMPLE_ADDR, env_port="")

    assert not ae.check_msg_cache(k1, k2, k3)
    ae.update_msg_cache(k1, k2, k3)
    assert not ae.check_msg_cache(k1, k2, k3)

    ae = AnalyzerExecutorSingleton(stub_env=True, env_addr="", env_port=SAMPLE_PORT)

    assert not ae.check_msg_cache(k1, k2, k3)
    ae.update_msg_cache(k1, k2, k3)
    assert not ae.check_msg_cache(k1, k2, k3)


@pytest.mark.integration_test
def test_message_cache_redis(AnalyzerExecutorSingleton, random_word) -> None:
    ae = AnalyzerExecutorSingleton(stub_env=False)

    k1, k2, k3 = random_word(), random_word(), random_word()

    assert not ae.check_msg_cache(k1, k2, k3)
    ae.update_msg_cache(k1, k2, k3)
    assert ae.check_msg_cache(k1, k2, k3)
