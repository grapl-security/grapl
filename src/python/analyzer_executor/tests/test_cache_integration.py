import os
from copy import deepcopy
from typing import Mapping, Optional

import hypothesis
import pytest
from analyzer_executor_lib.analyzer_executor import AnalyzerExecutor
from hypothesis import strategies as st

SAMPLE_ADDR = "localhost"
SAMPLE_PORT = "12345"

NonemptyStringStrategy = st.text(min_size=3, max_size=64)


class AnalyzerExecutorCacheDeleters:
    def __init__(self, analyzer_executor: AnalyzerExecutor) -> None:
        self.analyzer_executor = analyzer_executor

    def delete_msg_cache(self, file: str, node_key: str, msg_id: str) -> None:
        event_hash = self.analyzer_executor.to_event_hash((file, node_key, msg_id))
        self.analyzer_executor.message_cache.delete(event_hash)

    def delete_hit_cache(self, file: str, node_key: str) -> None:
        event_hash = self.analyzer_executor.to_event_hash((file, node_key))
        self.analyzer_executor.hit_cache.delete(event_hash)


def fake_os_env(
    env_addr: Optional[str] = None,
    env_port: Optional[str] = None,
) -> Mapping[str, str]:
    new_os_environ = deepcopy(os.environ)
    if env_addr:
        new_os_environ["MESSAGECACHE_ADDR"] = env_addr
        new_os_environ["HITCACHE_ADDR"] = env_addr
    else:
        del new_os_environ["MESSAGECACHE_ADDR"]
        del new_os_environ["HITCACHE_ADDR"]

    if env_port:
        new_os_environ["MESSAGECACHE_PORT"] = env_port
        new_os_environ["HITCACHE_PORT"] = env_port
    else:
        del new_os_environ["MESSAGECACHE_PORT"]
        del new_os_environ["HITCACHE_PORT"]
    return new_os_environ


@pytest.mark.integration_test
def test_connection_info() -> None:
    """
    Ensures exceptions are raised for incomplete connection info.
    """

    with pytest.raises(ValueError):
        ae = AnalyzerExecutor.from_env(fake_os_env(env_addr=SAMPLE_ADDR, env_port=None))

    with pytest.raises(ValueError):
        ae = AnalyzerExecutor.from_env(fake_os_env(env_addr=None, env_port=SAMPLE_PORT))

    with pytest.raises(ValueError):
        ae = AnalyzerExecutor.from_env(fake_os_env(env_addr=None, env_port=None))


@hypothesis.given(
    k1=NonemptyStringStrategy,
    k2=NonemptyStringStrategy,
)
@hypothesis.settings(
    # Doesn't like the Pytest fixture mixed with Hypothesis givens.
    # It's okay, since the fixture just returns a function.
    suppress_health_check=[hypothesis.HealthCheck.function_scoped_fixture],
    deadline=None,
)
@pytest.mark.integration_test
def test_hit_cache(
    k1: str,
    k2: str,
) -> None:
    """
    Initializes the AnalyzerExecutor singleton with Redis connection params
    sourced from the environment, expecting hit cache to populate.
    """
    ae = AnalyzerExecutor.from_env()

    assert not ae.check_hit_cache(k1, k2)
    ae.update_hit_cache(k1, k2)
    assert ae.check_hit_cache(k1, k2)
    # Clean up, because Hypothesis provides duplicate inputs
    AnalyzerExecutorCacheDeleters(ae).delete_hit_cache(k1, k2)
    assert not ae.check_hit_cache(k1, k2)


@hypothesis.given(
    k1=NonemptyStringStrategy,
    k2=NonemptyStringStrategy,
    k3=NonemptyStringStrategy,
)
@hypothesis.settings(
    # Doesn't like the Pytest fixture mixed with Hypothesis givens.
    # It's okay, since the fixture just returns a function.
    suppress_health_check=[hypothesis.HealthCheck.function_scoped_fixture],
    deadline=None,
)
@pytest.mark.integration_test
def test_message_cache(
    k1: str,
    k2: str,
    k3: str,
) -> None:
    """
    Initializes the AnalyzerExecutor singleton with Redis connection params
    sourced from the environment, expecting message cache to populate.
    """
    ae = AnalyzerExecutor.from_env()

    assert not ae.check_msg_cache(k1, k2, k3)
    ae.update_msg_cache(k1, k2, k3)
    assert ae.check_msg_cache(k1, k2, k3)
    # Clean up, because Hypothesis provides duplicate inputs
    AnalyzerExecutorCacheDeleters(ae).delete_msg_cache(k1, k2, k3)
    assert not ae.check_msg_cache(k1, k2, k3)
