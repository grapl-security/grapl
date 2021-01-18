import json
import time
import traceback

import boto3  # type: ignore
import botocore.exceptions  # type: ignore
from analyzer_executor_lib.analyzer_executor import LOGGER, AnalyzerExecutor
from analyzer_executor_lib.event_retriever import s3_event_retrieve
from grapl_common.debugger.vsc_debugger import wait_for_vsc_debugger
from grapl_common.env_helpers import SQSClientFactory

ANALYZER_EXECUTOR = AnalyzerExecutor.singleton()

wait_for_vsc_debugger(service="analyzer_executor")


def main():
    for sqs_message_body in s3_event_retrieve():
        ANALYZER_EXECUTOR.lambda_handler_fn(sqs_message_body, {})


main()
