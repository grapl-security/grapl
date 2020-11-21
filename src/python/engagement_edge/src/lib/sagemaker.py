from __future__ import annotations

from typing import TYPE_CHECKING

import boto3

from src.lib.env_vars import IS_LOCAL

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
    def create() -> SagemakerClient:
        if IS_LOCAL:
            # Technically, we could put in some mock that just returns the Jupyter url, but
            # at that point you're getting pretty far from the AWS implementation.
            raise NotImplementedError("There's no localstack Sagemaker yet!")

        client = boto3.client("sagemaker")
        return SagemakerClient(client=client)
