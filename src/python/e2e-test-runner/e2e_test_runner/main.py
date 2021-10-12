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

    _cd_to_test_directory()
    result = setup_tests.exec_pytest()

    LOGGER.info(f"tests completed with status code {result}")

    sys.exit(result)


def _cd_to_test_directory() -> None:
    """
    One intricacy of running Pytest in a PEX is that the PEX format messes with
    pytest and xunit's test discovery (since that often depends on file structure).
    As such, we just `chdir` into the site-packages folder for `e2e_test_runner`.
    While there are potentially more robust ways to do this, this one works.
    """
    this_file = Path(__file__).resolve()
    site_packages = next(p for p in this_file.parents if p.name == "site-packages")
    os.chdir(site_packages / "e2e_test_runner")
