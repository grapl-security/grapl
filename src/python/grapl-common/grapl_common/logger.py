import logging
import os
from typing import Any

import structlog

StructuredLogger = Any  # Unfortunately, `structlog.get_logger()` just returns an Any


def get_structlogger() -> StructuredLogger:
    log_level_name = os.environ["GRAPL_LOG_LEVEL"]  # e.g. "DEBUG"
    log_level: int = getattr(logging, log_level_name)  # e.g. logging.DEBUG, an int
    structlog.configure(
        # Output as JSON
        processors=[structlog.processors.JSONRenderer()],
        # Filters out logs with a too-low log level like the built-in py logger
        wrapper_class=structlog.make_filtering_bound_logger(
            min_level=log_level,
        ),
    )
    return structlog.get_logger()
