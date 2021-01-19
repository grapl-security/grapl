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
    """
    This will eventually become the basis of the Python equivalent of `process_loop()`.
    Some TODOs:
    - make sure SOURCE_QUEUE_URL is also specified in CDK
    - add
      RETRY_QUEUE_URL
      DEAD_LETTER_QUEUE_URL
      DEST_QUEUE_URL
    - pull the manual eventing out of `lambda_handler_fn` and into an EventEmitter
    """
    queue_url = os.environ["SOURCE_QUEUE_URL"]
    retriever = EventRetriever(queue_url=queue_url)
    for sqs_message_body in retriever.retrieve():
        ANALYZER_EXECUTOR.lambda_handler_fn(sqs_message_body, {})


main()
