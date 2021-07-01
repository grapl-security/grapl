import base64
from typing import cast, TYPE_CHECKING

from botocore.response import StreamingBody
from grapl_common.grapl_logger import get_module_grapl_logger
from grapl_common.resources import known_dynamodb_tables

from mypy_boto3_dynamodb.service_resource import DynamoDBServiceResource, Table
from mypy_boto3_lambda import LambdaClient
from mypy_boto3_lambda.type_defs import InvocationResponseTypeDef

LOGGER = get_module_grapl_logger(log_to_stdout=True)


def _extract_invocation_response_error_payload(
    result: InvocationResponseTypeDef,
) -> str:
    """extract the payload of a lambda invocation error response in
    a format amenable to logging"""
    return "\\n".join(
        l.decode("utf-8") for l in cast(StreamingBody, result["Payload"]).iter_lines()
    )


def _invoke_lambda(lambda_: LambdaClient, function_name: str) -> None:
    LOGGER.info(f"invoking lambda {function_name}")
    result = lambda_.invoke(
        FunctionName=function_name,
        InvocationType="RequestResponse",
        LogType="Tail",
    )

    status = result["StatusCode"]
    logs = base64.b64decode(bytes(result["LogResult"], "utf-8")).decode("utf-8")
    if status == 200 and result.get("FunctionError") is None:
        for line in logs.splitlines():
            LOGGER.info(line)
        LOGGER.info(f"lambda invocation succeeded for {function_name}")
    else:
        LOGGER.error(
            f"lambda invocation for {function_name} returned error response {_extract_invocation_response_error_payload(result)}"
        )
        raise Exception(
            f"lambda invocation for {function_name} failed with status {status}: {result['FunctionError']}"
        )


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


def provision_grapl(lambda_: LambdaClient, deployment_name: str) -> None:
    _invoke_lambda(lambda_=lambda_, function_name=f"{deployment_name}-provisioner")


def run_e2e_tests(lambda_: LambdaClient, deployment_name: str) -> None:
    _invoke_lambda(lambda_=lambda_, function_name=f"{deployment_name}-e2e-test-runner")


def wipe_dynamodb(dynamodb: DynamoDBServiceResource, deployment_name: str) -> None:
    session_table = known_dynamodb_tables.session_table(dynamodb, deployment_name)
    schema_table = known_dynamodb_tables.schema_table(dynamodb, deployment_name)
    schema_properties_table = known_dynamodb_tables.schema_properties_table(
        dynamodb, deployment_name
    )
    for table in (session_table, schema_table, schema_properties_table):
        LOGGER.info(f"Wiping {table}")
        _wipe_dynamodb_table(table)
        LOGGER.info(f"Wiped {table}")
