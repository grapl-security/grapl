import json
import os
import time
import traceback

import boto3  # type: ignore
import botocore.exceptions  # type: ignore
from analyzer_executor_lib.analyzer_executor import LOGGER, AnalyzerExecutor
from analyzer_executor_lib.event_retriever import EventRetriever
from grapl_common.debugger.vsc_debugger import wait_for_vsc_debugger
from grapl_common.env_helpers import SQSClientFactory

ANALYZER_EXECUTOR = AnalyzerExecutor.singleton()

wait_for_vsc_debugger(service="analyzer_executor")


def main():
    # TODO make sure this is also specified in the cdk
    queue_url = os.environ["SOURCE_QUEUE_URL"]
    # RETRY_QUEUE_URL
    # DEAD_LETTER_QUEUE_URL
    # DEST_QUEUE_URL
    retriever = EventRetriever(queue_url=queue_url)
    for sqs_message_body in s3_event_retrieve():
        ANALYZER_EXECUTOR.lambda_handler_fn(sqs_message_body, {})


main()
