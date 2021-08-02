from grapl_common.grapl_logger import get_module_grapl_logger
from mypy_boto3_dynamodb.service_resource import DynamoDBServiceResource, Table

LOGGER = get_module_grapl_logger(log_to_stdout=True)


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
