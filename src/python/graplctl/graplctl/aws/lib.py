from __future__ import annotations

from typing import TYPE_CHECKING

from grapl_common.grapl_logger import get_module_grapl_logger

if TYPE_CHECKING:
    from mypy_boto3_dynamodb.service_resource import DynamoDBServiceResource, Table

LOGGER = get_module_grapl_logger(log_to_stdout=True)


def _wipe_dynamodb_table(table: Table) -> None:
    """
    Based off https://stackoverflow.com/a/61641725
    """
    # get the table keys
    table_key_names = [key.get("AttributeName") for key in table.key_schema]

    """
    NOTE: there are reserved attributes for key names, please see https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ReservedWords.html
    if a hash or range key is in the reserved word list, you will need to use the ExpressionAttributeNames parameter
    described at https://boto3.amazonaws.com/v1/documentation/api/latest/reference/services/dynamodb.html#DynamoDB.Table.scan
    """

    # Only retrieve the keys for each item in the table (minimize data transfer)
    projection_expression = ", ".join(table_key_names)

    response = table.scan(ProjectionExpression=projection_expression)
    data = response.get("Items")
    assert data is not None, f"Expected items, got {data}"

    while "LastEvaluatedKey" in response:
        response = table.scan(
            ProjectionExpression=projection_expression,
            ExclusiveStartKey=response["LastEvaluatedKey"],
        )
        data.extend(response["Items"])

    with table.batch_writer() as batch:
        for each in data:
            batch.delete_item(Key={key: each[key] for key in table_key_names})


def wipe_dynamodb(
    dynamodb: DynamoDBServiceResource,
    schema_table_name: str,
    schema_properties_table_name: str,
    dynamic_session_table_name: str,
) -> None:
    session_table = dynamodb.Table(dynamic_session_table_name)
    schema_table = dynamodb.Table(schema_table_name)
    schema_properties_table = dynamodb.Table(schema_properties_table_name)
    for table in (session_table, schema_table, schema_properties_table):
        LOGGER.info(f"Wiping {table}")
        _wipe_dynamodb_table(table)
        LOGGER.info(f"Wiped {table}")
