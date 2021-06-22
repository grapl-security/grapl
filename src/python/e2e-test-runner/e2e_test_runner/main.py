import os
import shutil
import sys
from typing import Any

from grapl_common.debugger.vsc_debugger import wait_for_vsc_debugger
from grapl_common.grapl_logger import get_module_grapl_logger

LOGGER = get_module_grapl_logger()


def lambda_handler(event: Any, context: Any) -> None:
    wait_for_vsc_debugger("grapl_e2e_tests")
    LOGGER.info("staging schemas")
    if os.path.exists("/tmp/schemas"):
        shutil.rmtree("/tmp/schemas")
    os.mkdir("/tmp/schemas")
    shutil.copyfile("e2e_test_runner/schemas.py", "/tmp/schemas/schemas.py")
    LOGGER.info("staged schemas")

    LOGGER.info("executing pytest")

    from grapl_tests_common import setup_tests  # import here to limit monkeypatch

    result = setup_tests.exec_pytest()

    LOGGER.info(f"tests completed with status code {result}")

    if result == 0:
        return  # Lambda runtime will flip out if sys.exit(0) -_-
    else:
        sys.exit(result)
