import base64
import logging
import os
import sys
from typing import cast

from botocore.response import StreamingBody
from mypy_boto3_lambda import LambdaClient
from mypy_boto3_lambda.type_defs import InvocationResponseTypeDef

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(os.getenv("GRAPL_LOG_LEVEL", "INFO"))
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

EDGE_UX_DIRECTORY = "/edge_ux_post_replace"
CDK_OUT_FILENAME = "cdk-output.json"


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


def provision_grapl(lambda_: LambdaClient, deployment_name: str) -> None:
    _invoke_lambda(lambda_=lambda_, function_name=f"{deployment_name}-provisioner")


def run_e2e_tests(lambda_: LambdaClient, deployment_name: str) -> None:
    _invoke_lambda(lambda_=lambda_, function_name=f"{deployment_name}-e2e-test-runner")
