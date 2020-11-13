import unittest
import pytest

from analyzer_executor_lib.analyzer_executor import AnalyzerExecutor

SAMPLE_ADDR = "localhost"
SAMPLE_PORT = "12345"

from hypothesis import strategies as st


@pytest.fixture
def AnalyzerExecutorSingleton(monkeypatch):
    def _AnalyzerExecutorSingleton(stub_env=False, env_addr=None, env_port=None):
        with monkeypatch.context() as mp:
            if stub_env:
                if env_addr:
                    mp.setenv("MESSAGECACHE_ADDR", env_addr)
                    mp.setenv("HITCACHE_ADDR", env_addr)
                else:
                    mp.delenv("MESSAGECACHE_ADDR", raising=False)
                    mp.delenv("HITCACHE_ADDR", raising=False)

                if env_port:
                    mp.setenv("MESSAGECACHE_PORT", env_port)
                    mp.setenv("HITCACHE_PORT", env_port)
                else:
                    mp.delenv("MESSAGECACHE_PORT", raising=False)
                    mp.delenv("HITCACHE_PORT", raising=False)

            # force singleton to reinitialize,
            # this should be idempotent?
            AnalyzerExecutor._singleton = None
            return AnalyzerExecutor.singleton()

    return _AnalyzerExecutorSingleton


@pytest.mark.integration_test
def test_connection_info(AnalyzerExecutorSingleton) -> None:
    """
    Ensures exceptions are raised for incomplete connection info.
    """

    with pytest.raises(ValueError):
        ae = AnalyzerExecutorSingleton(
            stub_env=True, env_addr=SAMPLE_ADDR, env_port=None
        )

    with pytest.raises(ValueError):
        ae = AnalyzerExecutorSingleton(
            stub_env=True, env_addr=None, env_port=SAMPLE_PORT
        )

    with pytest.raises(ValueError):
        ae = AnalyzerExecutorSingleton(stub_env=True, env_addr=None, env_port=None)


@pytest.mark.integration_test
def test_hit_cache(AnalyzerExecutorSingleton) -> None:
    """
    Initializes the AnalyzerExecutor singleton with Redis connection params
    sourced from the environment, expecting hit cache to populate.
    """
    ae = AnalyzerExecutorSingleton(stub_env=False)

    k1, k2 = st.text(min_size=3), st.text(min_size=3)

    assert not ae.check_hit_cache(k1, k2)
    ae.update_hit_cache(k1, k2)
    assert ae.check_hit_cache(k1, k2)


@pytest.mark.integration_test
def test_message_cache(AnalyzerExecutorSingleton) -> None:
    """
    Initializes the AnalyzerExecutor singleton with Redis connection params
    sourced from the environment, expecting message cache to populate.
    """
    ae = AnalyzerExecutorSingleton(stub_env=False)

    k1, k2, k3 = st.text(min_size=3), st.text(min_size=3), st.text(min_size=3)

    assert not ae.check_msg_cache(k1, k2, k3)
    ae.update_msg_cache(k1, k2, k3)
    assert ae.check_msg_cache(k1, k2, k3)
