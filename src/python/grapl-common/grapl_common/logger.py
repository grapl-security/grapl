import logging
import os

import structlog


def get_structlogger() -> structlog.BoundLogger:
    log_level_name = os.environ["GRAPL_LOG_LEVEL"]  # e.g. "DEBUG"
    log_level: int = getattr(logging, log_level_name)  # e.g. logging.DEBUG, an int
    structlog.configure(
        processors=[
            # include {"level": "INFO"} in the dict
            structlog.processors.add_log_level,
            # Output as JSON
            structlog.processors.JSONRenderer(),
        ],
        # Filters out logs with a too-low log level like the built-in py logger
        wrapper_class=structlog.make_filtering_bound_logger(
            min_level=log_level,
        ),
        context_class=dict,
    )
    return structlog.stdlib.get_logger()
