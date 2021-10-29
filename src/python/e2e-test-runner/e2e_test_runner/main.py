import os
import sys
from pathlib import Path

from grapl_common.debugger.vsc_debugger import wait_for_vsc_debugger
from grapl_common.grapl_logger import get_module_grapl_logger

LOGGER = get_module_grapl_logger()


def main() -> None:
    wait_for_vsc_debugger("grapl_e2e_tests")

    LOGGER.info("executing pytest")

    from grapl_tests_common import setup_tests  # import here to limit monkeypatch

    # Change to the parent directory so pytest can find the tests
    os.chdir(Path(__file__).resolve().parent)
    result = setup_tests.exec_pytest()

    LOGGER.info(f"tests completed with status code {result}")

    sys.exit(result)
