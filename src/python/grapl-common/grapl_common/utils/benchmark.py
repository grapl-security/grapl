from contextlib import contextmanager
from datetime import datetime, timedelta
from typing import Iterator, Optional


class BenchmarkResult:
    result: Optional[timedelta]

    def __init__(self) -> None:
        self.result = None

    def __str__(self) -> str:
        assert self.result, "You can't consume BenchmarkResult until after the context."
        return f"{int(self.result.total_seconds())} seconds"


@contextmanager
def benchmark_ctx() -> Iterator[BenchmarkResult]:
    will_contain_result = BenchmarkResult()
    start = datetime.utcnow()
    yield will_contain_result
    end = datetime.utcnow()

    result = end - start
    will_contain_result.result = result
