from time import sleep
import logging
import boto3
from typing import Any, Set, Sequence
from typing_extensions import Protocol
from itertools import cycle


class WaitForResource(Protocol):
    def exists(self) -> bool:
        pass


class WaitForS3Bucket(WaitForResource):
    def __init__(self, s3_client: Any, bucket_name: str):
        self.s3_client = s3_client
        self.bucket_name = bucket_name

    def exists(self) -> bool:
        try:
            self.s3_client.head_bucket(Bucket=self.bucket_name)
            return True
        except self.s3_client.exceptions.NoSuchBucket:
            return False

    def __str__(self) -> str:
        return f"WaitForS3Bucket({self.bucket_name})"


class WaitForSqsQueue(WaitForResource):
    def __init__(self, sqs_client: Any, queue_name: str):
        self.sqs_client = sqs_client
        self.queue_name = queue_name

    def exists(self) -> bool:
        try:
            self.sqs_client.get_queue_url(QueueName=self.queue_name)
            return True
        except self.sqs_client.exceptions.QueueDoesNotExist:
            return False

    def __str__(self) -> str:
        return f"WaitForSqsQueue({self.queue_name})"


def wait_on_resources(resources: Sequence[WaitForResource]):
    completed: Set[WaitForResource] = set()

    # Cycle through `resources` forever, until all resources are attained
    # hacky? potentially O(infinity)? yes
    for resource in cycle(resources):
        if len(completed) == len(resources):
            break
        if resource in completed:
            continue

        logging.info(f"Checking resource: {resource}")

        now_exists = resource.exists()
        if now_exists:
            completed.add(resource)
        sleep(1)
