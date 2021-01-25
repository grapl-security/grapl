# (generated with --quick)

from typing import Callable, Sequence, Type, TypeVar

time: module

F = TypeVar("F", bound=Callable)

def retry(
    ExceptionToCheck: Type[Exception] = ...,
    on_falsey: bool = ...,
    tries: int = ...,
    delay: float = ...,
    backoff: int = ...,
) -> Callable[[F], F]: ...
def wraps(
    wrapped: Callable, assigned: Sequence[str] = ..., updated: Sequence[str] = ...
) -> Callable[[Callable], Callable]: ...
