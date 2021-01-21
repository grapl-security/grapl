import asyncio
import os

import boto3
from analyzer_executor_lib.analyzer_executor import AnalyzerExecutor
from analyzer_executor_lib.event_retriever import EventRetriever
from analyzer_executor_lib.grapl_logger import get_module_grapl_logger
from analyzer_executor_lib.sqs_timeout_manager import (
    SqsTimeoutManager,
    SqsTimeoutManagerException,
)
from grapl_common.debugger.vsc_debugger import wait_for_vsc_debugger
from grapl_common.env_helpers import SQSClientFactory

wait_for_vsc_debugger(service="analyzer_executor")

ANALYZER_EXECUTOR = AnalyzerExecutor.singleton()
LOGGER = get_module_grapl_logger()


async def main():
    """
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
    sqs_client = SQSClientFactory(boto3).from_env()
    for (sqs_message_body, sqs_receipt_handle) in retriever.retrieve():
        # We'd feed this coroutine into the timeout manager.
        message_handle_coroutine = ANALYZER_EXECUTOR.lambda_handler_fn(
            sqs_message_body, {}
        )
        # While we're waiting for that future to complete, keep telling SQS
        # "hey, we're working on it" so it doesn't become visible on the
        # input queue again.
        timeout_manager = SqsTimeoutManager(
            sqs_client=sqs_client,
            queue_url=queue_url,
            receipt_handle=sqs_receipt_handle,
            message_id="todo",
            coroutine=message_handle_coroutine,
        )
        try:
            await timeout_manager.keep_alive()
        except SqsTimeoutManagerException:
            LOGGER.error("SQS Timeout Manager exception", exc_info=True)


asyncio.run(main())
