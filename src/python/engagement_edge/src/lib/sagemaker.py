from __future__ import annotations

from typing import TYPE_CHECKING

import boto3

if TYPE_CHECKING:
    from mypy_boto3_sagemaker import (
        SageMakerClient as _BotoSageMakerClient,  # not a fan of that capital M
    )


class SagemakerClient:
    def __init__(self, client: _BotoSageMakerClient):
        self.client = client

    def get_presigned_url(self, instance_name: str) -> str:
        result = self.client.create_presigned_notebook_instance_url(
            NotebookInstanceName=instance_name,
            # defaults to 12 hours
        )
        return result["AuthorizedUrl"]

    @staticmethod
    def create(is_local: bool) -> SagemakerClient:
        client = boto3.client("sagemaker")
        if is_local:
            return LocalSagemakerClient(client=client)
        else:
            return SagemakerClient(client=client)


class LocalSagemakerClient(SagemakerClient):
    def get_presigned_url(self, instance_name: str) -> str:
        return "http://localhost:8888"
