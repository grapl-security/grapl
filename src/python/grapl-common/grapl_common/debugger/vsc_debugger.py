"""
This is reasonably well-documented in `vscode_debugger.rst`.

I arbitrarily chose `debugpy`, the VS Code debugger (formerly known as ptsvd).
It'd be pretty easy to add the PyCharm/IntelliJ debugger too (which uses pydevd)
"""
import os
import subprocess
import sys
from typing import Optional

from grapl_common.grapl_logger import get_module_grapl_logger
from typing_extensions import Literal

LOGGER = get_module_grapl_logger()

ServiceIdentifier = Literal[
    "grapl_e2e_tests",
    "analyzer_executor",
    "graphql_endpoint_tests",
]


def _install_from_pip(package: str) -> None:
    # Gross, but the suggested way to install from pip mid-python-program!
    # Doing it this way so that <every executable that depends on grapl_common> doesn't inherently
    # need to import debugpy
    subprocess.check_call(
        [sys.executable, "-m", "pip", "install", "--upgrade", package]
    )


def _should_debug_service(service: ServiceIdentifier) -> bool:
    """
    When you set
    DEBUG_SERVICES=grapl_e2e_tests
    you'll start a debug listener on the `grapl_e2e_tests`

    When you set
    DEBUG_SERVICES=grapl_e2e_tests,some_future_service
    you'll start two debug listeners - 1 for each service - on different ports!
    """
    env_var = os.getenv("DEBUG_SERVICES")
    if not env_var:
        return False
    debug_services = set(env_var.split(","))
    return service in debug_services


def _get_debug_port() -> Optional[int]:
    port = os.getenv("VSC_DEBUGGER_PORT")
    if not port:
        return None
    else:
        port_int = int(port)
        assert (
            8400 <= port_int < 8500
        ), "84xx range is reserved for our debuggers. You likely want 1 per service."
        return port_int


def wait_for_vsc_debugger(service: ServiceIdentifier) -> None:
    if not _should_debug_service(service):
        return

    port = _get_debug_port()
    if port is None:
        LOGGER.error(f"Couldn't find a debug port for service {service}.")
        return

    _install_from_pip("debugpy")
    import debugpy  # type: ignore

    host = "0.0.0.0"
    LOGGER.info(f">> Debugpy listening for client at {host}:{port}")
    debugpy.listen((host, port))
    debugpy.wait_for_client()
    LOGGER.info(f">> Debugpy connected!")
