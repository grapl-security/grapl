from __future__ import annotations

from typing import (
    TYPE_CHECKING,
    Any,
    Callable,
    Dict,
    List,
    Optional,
    Tuple,
    TypeVar,
    Union,
    cast,
)

import boto3
from chalice import Response
from src.lib.constants import IS_LOCAL

if TYPE_CHECKING:
    from mypy_boto3_sagemaker import (
        SageMakerClient as SagemakerClient,  # not a fan of that capital M
    )


class SagemakerNotebookUrlGetter:
    def __init__(self, client: SagemakerClient):
        self.client = client

    def get_presigned_url(self, instance_name: str) -> str:
        result = self.client.create_presigned_notebook_instance_url(
            NotebookInstanceName=instance_name,
            # defaults to 12 hours
        )
        return result["AuthorizedUrl"]

    @staticmethod
    def create() -> SagemakerNotebookUrlGetter:
        if IS_LOCAL:
            # Technically, we could put in some mock that just returns the Jupyter url, but
            # at that point you're getting pretty far from the AWS implementation.
            raise NotImplementedError("There's no localstack Sagemaker yet!")

        client = boto3.client("sagemaker")
        return SagemakerNotebookUrlGetter(client=client)
