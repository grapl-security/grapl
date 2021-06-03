import os
import shutil
from typing import Any

from grapl_common.debugger.vsc_debugger import wait_for_vsc_debugger
from grapl_tests_common import setup_tests


def lambda_handler(event: Any, context: Any) -> None:
    wait_for_vsc_debugger("grapl_e2e_tests")
    if os.path.exists("/tmp/schemas"):
        shutil.rmtree("/tmp/schemas")
    os.mkdir("/tmp/schemas")
    shutil.copyfile("e2e_test_runner/schemas.py", "/tmp/schemas/schemas.py")
    setup_tests.exec_pytest()
