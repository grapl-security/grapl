from __future__ import annotations

from typing import TYPE_CHECKING
from typing_extensions import Protocol

import boto3

if TYPE_CHECKING:
    from mypy_boto3_sagemaker import (
        SageMakerClient as _BotoSageMakerClient,  # not a fan of that capital M
    )


class ISagemakerClient(Protocol):
    def get_presigned_url(self, instance_name: str) -> str:
        pass


class SagemakerClient(ISagemakerClient):
    def __init__(self, client: _BotoSageMakerClient):
        self.client = client

    def get_presigned_url(self, instance_name: str) -> str:
        result = self.client.create_presigned_notebook_instance_url(
            NotebookInstanceName=instance_name,
            # defaults to 12 hours
        )
        return result["AuthorizedUrl"]


class LocalSagemakerClient(ISagemakerClient):
    def get_presigned_url(self, instance_name: str) -> str:
        return "http://localhost:8888"


def create_sagemaker_client(is_local: bool) -> ISagemakerClient:
    if is_local:
        return LocalSagemakerClient()
    else:
        client = boto3.client("sagemaker")
        return SagemakerClient(client=client)
