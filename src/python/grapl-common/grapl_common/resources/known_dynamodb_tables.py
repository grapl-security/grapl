"""
In general, we'll want to depend on table names injected with `os.environ`; but
some instances (especially graplctl) still need the old string-formatting way.
"""

from __future__ import annotations

import os
from typing import TYPE_CHECKING, Optional

if TYPE_CHECKING:
    from mypy_boto3_dynamodb import DynamoDBServiceResource
    from mypy_boto3_dynamodb.service_resource import Table


def _get_table(
    dynamodb: DynamoDBServiceResource,
    suffix: str,
    deployment_name: Optional[str] = None,
) -> Table:
    deployment_name = deployment_name or os.environ["DEPLOYMENT_NAME"]
    return dynamodb.Table(f"{deployment_name}{suffix}")


def session_table(
    dynamodb: DynamoDBServiceResource, deployment_name: Optional[str] = None
) -> Table:
    return _get_table(
        dynamodb, "-dynamic_session_table", deployment_name=deployment_name
    )
