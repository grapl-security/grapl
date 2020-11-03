"""
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
    # Make sure you expose the port in dobi.yaml or docker-compose.yaml.
    "grapl_e2e_tests": 8400,
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


"""
Add the following as a `launch.json` debug configuration in VSCode.
You'll want a different configuration for each service you want to debug.
As such, each configuration should likely have a different path-mapping and a different port.

{
    "version": "0.2.0",
    "configurations": [
        {
            "name": "E2E tests debug",
            "type": "python",
            "request": "attach",
            "connect": {
                "host": "127.0.0.1",
                "port": 8400
            },
            // Also debug library code, like grapl-tests-common
            "justMyCode": false,
            "pathMappings": [
                {
                    "localRoot": "${workspaceFolder}/src/python/grapl_e2e_tests",
                    "remoteRoot": "/home/grapl/grapl_e2e_tests"
                },
                {
                    "localRoot": "${workspaceFolder}/src/python/grapl-tests-common/grapl_tests_common",
                    "remoteRoot": "../venv/lib/python3.7/site-packages/grapl_tests_common"
                },
                {
                    "localRoot": "${workspaceFolder}/src/python/grapl_analyzerlib/grapl_analyzerlib",
                    "remoteRoot": "../venv/lib/python3.7/site-packages/grapl_analyzerlib"
                }
            ]
        }
    ]
}
"""
