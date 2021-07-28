import logging
import time
from functools import wraps
from typing import Any, Callable, Type, TypeVar, cast

F = TypeVar("F", bound=Callable)


def retry(
    exception_cls: Type[Exception],
    logger: logging.Logger,
    on_falsey: bool = False,
    tries: int = 3,
    delay: float = 0.5,
    backoff: int = 2,
) -> Callable[[F], F]:
    """Retry calling the decorated function using an exponential backoff.

    Modified to handle 'falsey' cases with a retry

    Inspired by:
    http://www.saltycrane.com/blog/2009/11/trying-out-retry-decorator-python/
    original from: http://wiki.python.org/moin/PythonDecoratorLibrary#Retry

    :param exception_cls: the exception to check. may be a tuple of
        exceptions to check
    :param on_falsey: Check if result is falsey, and retry if it is

    :param tries: number of times to try (not retry) before giving up

    :param delay: initial delay between retries in seconds

    :param backoff: backoff multiplier e.g. value of 2 will double the delay
        each retry
    """

    def deco_retry(f: F) -> F:
        @wraps(f)
        def f_retry(*args: Any, **kwargs: Any) -> Any:
            mtries, mdelay = tries, delay
            while mtries > 1:

                try:
                    result = f(*args, **kwargs)

                    if on_falsey and not result:
                        time.sleep(mdelay)
                    else:
                        logger.debug(f"{retry_label} success")
                        return result
                except exception_cls as e:
                    iteration = tries - mtries + 1
                    retry_label = f"@retry: {iteration}/{tries}"
                    logger.debug(f"{retry_label} failed due to {e}")
                    time.sleep(mdelay)
                finally:
                    mtries -= 1
                    mdelay *= backoff

            return f(*args, **kwargs)

        return cast(F, f_retry)  # true decorator

    return deco_retry
