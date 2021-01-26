import inspect
import logging
import os


def get_module_grapl_logger() -> logging.Logger:
    """
    Callers should put
    LOGGER = get_module_grapl_logger()
    at module scope.
    """
    caller_stack = inspect.stack()[1]
    caller_module = inspect.getmodule(caller_stack[0])
    assert caller_module
    logger = logging.getLogger(caller_module.__name__)
    logger.setLevel(os.getenv("GRAPL_LOG_LEVEL", "ERROR"))
    # While a lot of our code defines this, I believe it just doubles our log output
    # logger.addHandler(logging.StreamHandler(stream=sys.stdout))
    return logger
