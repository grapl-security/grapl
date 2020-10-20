from datetime import datetime, timezone, timedelta
from grapl_analyzerlib.grapl_client import MasterGraphClient
from grapl_analyzerlib.nodes.base import BaseView, BaseQuery
from grapl_analyzerlib.retry import retry
from itertools import cycle
from time import sleep
from typing_extensions import Protocol
from typing import Any, Sequence, Optional, Dict, Mapping, Callable
import botocore  # type: ignore
import inspect
import logging


class WaitForResource(Protocol):
    def acquire(self) -> Optional[Any]:
        pass


class WaitForS3Bucket(WaitForResource):
    def __init__(self, s3_client: Any, bucket_name: str):
        self.s3_client = s3_client
        self.bucket_name = bucket_name

    def acquire(self) -> Optional[Any]:
        try:
            return self.s3_client.head_bucket(Bucket=self.bucket_name)
        except self.s3_client.exceptions.NoSuchBucket:
            return None

    def __str__(self) -> str:
        return f"WaitForS3Bucket({self.bucket_name})"


class WaitForSqsQueue(WaitForResource):
    def __init__(self, sqs_client: Any, queue_name: str):
        self.sqs_client = sqs_client
        self.queue_name = queue_name

    def acquire(self) -> Optional[Any]:
        try:
            return self.sqs_client.get_queue_url(QueueName=self.queue_name)
        except (
            self.sqs_client.exceptions.QueueDoesNotExist,
            botocore.parsers.ResponseParserError,
        ):
            return None

    def __str__(self) -> str:
        return f"WaitForSqsQueue({self.queue_name})"


class WaitForCondition(WaitForResource):
    """
    Retry a Callable until it returns true
    """

    def __init__(self, fn: Callable[[], Optional[bool]]) -> None:
        self.fn = fn

    def acquire(self) -> Optional[Any]:
        result = self.fn()
        if result:
            return self  # just anything non-None
        else:
            return None

    def __str__(self) -> str:
        return f"WaitForCondition({inspect.getsource(self.fn)})"


class WaitForNoException(WaitForResource):
    """
    Retry a Callable until it stops throwing exceptions.
    """

    def __init__(self, fn: Callable) -> None:
        self.fn = fn

    def acquire(self) -> Optional[Any]:
        try:
            return self.fn()
        except:
            return None

    def __str__(self) -> str:
        return f"WaitForNoException({inspect.getsource(self.fn)})"


class WaitForQuery(WaitForResource):
    def __init__(self, query: BaseQuery, dgraph_client: Any = None) -> None:
        self.query = query
        self.dgraph_client = dgraph_client or MasterGraphClient()

    @retry()
    def acquire(self) -> Optional[BaseView]:
        result = self.query.query_first(self.dgraph_client)
        return result

    def __str__(self) -> str:
        return f"WaitForLens({self.query})"


def wait_for(
    resources: Sequence[WaitForResource],
    timeout_secs: int = 30,
    sleep_secs: int = 5,
) -> Mapping[WaitForResource, Any]:
    completed: Dict[WaitForResource, Any] = {}

    get_now = lambda: datetime.now(tz=timezone.utc)

    timeout_after = get_now() + timedelta(seconds=timeout_secs)

    # Cycle through `resources` forever, until all resources are attained
    # hacky? potentially O(infinity)? yes
    for resource in cycle(resources):
        now = get_now()
        if now >= timeout_after:
            raise TimeoutError(f"Timed out waiting for {resource}")
        if len(completed) == len(resources):
            break
        if resource in completed:
            continue

        secs_remaining = int((timeout_after - now).total_seconds())
        # print an update every 5 secs
        logging.info(f"Waiting for resource ({secs_remaining} secs remain): {resource}")

        result = resource.acquire()
        if result is not None:
            completed[resource] = result
        else:
            sleep(sleep_secs)

    return completed


def wait_for_one(one: WaitForResource, timeout_secs: int = 60) -> Any:
    results = wait_for([one], timeout_secs=timeout_secs)
    return results[one]
