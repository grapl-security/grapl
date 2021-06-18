from __future__ import annotations

import logging
from pathlib import Path
from pprint import pformat as pretty_format
from typing import TYPE_CHECKING, Optional

import boto3
from grapl_common.env_helpers import DynamoDBResourceFactory

if TYPE_CHECKING:
    from mypy_boto3_dynamodb.service_resource import DynamoDBServiceResource, Table

DOCKER_VOLUME = Path("/mnt/dynamodb_dump")


def dump_dynamodb() -> None:
    dynamodb: DynamoDBServiceResource = DynamoDBResourceFactory(boto3).from_env()
    logging.info("Dumping DynamoDB")

    tables = [x for x in dynamodb.tables.all()]
    for table in tables:
        table_dump = _dump_dynamodb_table(table)
        if not table_dump:
            logging.info(f"No items in {table.name}")
            continue

        path = DOCKER_VOLUME.joinpath(table.name).resolve()
        with open(path, "w+") as f:
            logging.info(f"Dumped {table.name} to Docker volume")
            f.write(table_dump)


def _dump_dynamodb_table(table: Table) -> Optional[str]:
    """
    Outputs a nicely-formatted Python list of all the items in the table.
    (you may need a `from decimal import Decimal` to interact with it, though.)
    """
    items = table.scan()["Items"]
    if not items:
        return None
    return pretty_format(items)
