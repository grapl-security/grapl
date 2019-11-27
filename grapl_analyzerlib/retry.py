import time
from functools import wraps


def retry(
    ExceptionToCheck: Exception = Exception,
    on_falsey: bool = True,
    tries: int = 3,
    delay: float = 0.5,
    backoff: int = 2,
):
    """Retry calling the decorated function using an exponential backoff.

    Modified to handle 'falsey' cases with a retry

    Inspired by:
    http://www.saltycrane.com/blog/2009/11/trying-out-retry-decorator-python/
    original from: http://wiki.python.org/moin/PythonDecoratorLibrary#Retry

    :param ExceptionToCheck: the exception to check. may be a tuple of
        exceptions to check
    :param on_falsey: Check if result is falsey, and retry if it is

    :param tries: number of times to try (not retry) before giving up

    :param delay: initial delay between retries in seconds

    :param backoff: backoff multiplier e.g. value of 2 will double the delay
        each retry
    """

    def deco_retry(f):
        @wraps(f)
        def f_retry(*args, **kwargs):
            mtries, mdelay = tries, delay
            while mtries > 1:
                try:
                    result = f(*args, **kwargs)

                    if on_falsey and not result:
                        time.sleep(mdelay)
                    else:
                        return result
                except ExceptionToCheck:
                    time.sleep(mdelay)
                finally:
                    mtries -= 1
                    mdelay *= backoff

            return f(*args, **kwargs)

        return f_retry  # true decorator

    return deco_retry
