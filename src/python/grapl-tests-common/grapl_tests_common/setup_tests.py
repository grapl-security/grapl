from __future__ import annotations

import os
from os import environ

import pytest
from grapl_common.logger import get_structlogger
from grapl_common.utils.primitive_convertors import to_bool
from grapl_tests_common.dump_dynamodb import dump_dynamodb

# Toggle if you want to dump databases, logs, etc.
DUMP_ARTIFACTS = to_bool(environ.get("DUMP_ARTIFACTS", False))

LOGGER = get_structlogger()
GRAPL_LOG_LEVEL = os.getenv("GRAPL_LOG_LEVEL", "INFO")


def _after_tests() -> None:
    """
    Add any "after tests are executed, but before docker compose down" stuff here.
    """
    if DUMP_ARTIFACTS:
        dump_dynamodb()


def exec_pytest() -> int:
    pytest_args: list[str] = []
    if environ.get("PYTEST_EXPRESSION"):
        pytest_args.extend(("-k", environ["PYTEST_EXPRESSION"]))

    result = pytest.main(
        [
            # Disables stdout capture. Different from `-rA` in that you can see
            # the output streaming in real time, as opposed to just reported
            # after the fact. This is convenient for the timeout-heavy e2e test
            "--capture=no",
            f"--log-level={GRAPL_LOG_LEVEL}",
            f"--log-cli-level={GRAPL_LOG_LEVEL}",
            *pytest_args,
        ]
    )
    _after_tests()

    return result
