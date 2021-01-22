"""
This is reasonably well-documented in `vscode_debugger.rst`.

I arbitrarily chose `debugpy`, the VS Code debugger (formerly known as ptsvd).
It'd be pretty easy to add the PyCharm/IntelliJ debugger too (which uses pydevd)
"""
import logging
import os
import subprocess
import sys


def _install_from_pip(package: str) -> None:
    # Gross, but the suggested way to install from pip mid-python-program!
    # Doing it this way so that <every executable that depends on grapl_common> doesn't inherently
    # need to import debugpy
    subprocess.check_call(
        [sys.executable, "-m", "pip", "install", "--upgrade", package]
    )


# TODO: we should probably have some fixed 'services' StrEnum somewhere
SERVICE_TO_PORT = {
    # As you need to debug more services, add more services here.
    # Make sure you expose the port in the appropriate docker-compose file.
    "grapl_e2e_tests": 8400,
    "analyzer_executor": 8401,
}


def _should_debug_service(service: str) -> bool:
    """
    When you set
    DEBUG_SERVICES=grapl_e2e_tests
    you'll start a debug listener on the `grapl_e2e_tests`

    When you set
    DEBUG_SERVICES=grapl_e2e_tests,some_future_service
    you'll start two debug listeners - 1 for each service
    """
    env_var = os.getenv("DEBUG_SERVICES")
    if not env_var:
        return False
    debug_services = set(env_var.split(","))
    return service in debug_services


def wait_for_vsc_debugger(service: str) -> None:
    if not _should_debug_service(service):
        return
    port = SERVICE_TO_PORT.get(service, None)
    if not port:
        logging.error("Couldn't find a debug port for service {service}.")
        return

    _install_from_pip("debugpy")
    import debugpy  # type: ignore

    assert (
        8400 <= port < 8500
    ), "84xx range is reserved for our debuggers. You likely want 1 per service."
    host = "0.0.0.0"
    logging.info(f">> Debugpy listening for client at {host}:{port}")
    debugpy.listen((host, port))
    debugpy.wait_for_client()
    logging.info(f">> Debugpy connected!")
