from time import sleep
import logging
import boto3
from typing import Any, Set
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
        except self.s3_client.exceptions.NoSuchBucket:
            return False
        else:
            return True

    def __str__(self) -> str:
        return f"WaitForS3Bucket({self.bucket_name})"


def wait_on_resources(s3_client: Any, bucket_prefix: str):
    # okay, so, as it turns out, these are instantly created, and this has no value-add
    # but I sort of like this pattern, so, eh
    resources = [
        WaitForS3Bucket(s3_client, f"{bucket_prefix}-analyzers-bucket"),
        WaitForS3Bucket(s3_client, f"{bucket_prefix}-sysmon-log-bucket"),
    ]

    completed: Set[WaitForResource] = set()

    # Cycle through `resources` forever, until all resources are attained
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
