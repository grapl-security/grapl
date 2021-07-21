import logging

import pytest


# Applies it to every test function automatically.
@pytest.fixture(scope="function", autouse=True)
def set_noisy_loggers_to_log_level_info(caplog):
    # We globally declare every logger should use DEBUG in `exec_pytest`,
    # and here we piecemeal set some of the less useful loggers to a
    # different level.

    # Ideally we'd be able to do this with a regex or something - I've opened a
    # discussion here: https://github.com/pytest-dev/pytest/discussions/8925
    logger_names = (
        "botocore.auth",
        "botocore.endpoint",
        "botocore.hooks",
        "botocore.loaders",
        "botocore.parsers",
        "botocore.retryhandler",
        "urllib3.connectionpool",
    )
    for logger_name in logger_names:
        caplog.set_level(logging.INFO, logger=logger_name)
