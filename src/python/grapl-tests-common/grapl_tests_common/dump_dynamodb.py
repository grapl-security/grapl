from __future__ import annotations

import logging
from pathlib import Path
from pprint import pformat as pretty_format
from typing import TYPE_CHECKING, Any, Optional

import boto3  # type: ignore

if TYPE_CHECKING:
    from mypy_boto3_dynamodb.service_resource import DynamoDBServiceResource

DOCKER_VOLUME = Path("/mnt/dynamodb_dump")


def dump_dynamodb() -> None:
    dynamodb: DynamoDBServiceResource = boto3.resource(
        "dynamodb",
        region_name="us-west-2",
        endpoint_url="http://dynamodb:8000",
        aws_access_key_id="dummy_cred_aws_access_key_id",
        aws_secret_access_key="dummy_cred_aws_secret_access_key",
    )
    logging.info("Dumping DynamoDB")

    tables = [x for x in dynamodb.tables.all()]
    for table in tables:
        table_dump = _dump_dynamodb_table(table)
        if not table_dump:
            logging.info(f"No items in {table.name}")
            continue

        filename = f"{table.name}.json"
        path = DOCKER_VOLUME.joinpath(filename).resolve()
        with open(path, "w+") as f:
            logging.info(f"Dumped {table.name} to Docker volume")
            f.write(table_dump)


def _dump_dynamodb_table(table: Any) -> Optional[str]:
    if not table.item_count:
        return None
    items = table.scan()["Items"]
    return pretty_format(items)
