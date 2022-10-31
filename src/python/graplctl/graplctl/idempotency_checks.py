from __future__ import annotations

from typing import TYPE_CHECKING

from grapl_common.logger import get_structlogger

if TYPE_CHECKING:
    from mypy_boto3_dynamodb.service_resource import (
        DynamoDBServiceResource,  # pants: no-infer-dep
    )
    from mypy_boto3_dynamodb.service_resource import Table  # pants: no-infer-dep

LOGGER = get_structlogger()


def is_grapl_provisioned(
    dynamodb: DynamoDBServiceResource,
    schema_table: str,
) -> bool:
    """
    We are doing a very simple check - "is there >0 things in schema_table?"
    to infer whether any provisioning has taken place.
    """
    table = dynamodb.Table(schema_table)
    return not _table_is_empty(table)


def _table_is_empty(table: Table) -> bool:
    """
    fun fact: some_table.item_count? It's only updated every 6 hours.
    you have to do a scan.
    """
    items = table.scan()["Items"]
    if items:
        return False  # Something's in there!
    return True
