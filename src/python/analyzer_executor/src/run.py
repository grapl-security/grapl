import asyncio
import os

import boto3
from analyzer_executor_lib.analyzer_executor import AnalyzerExecutor
from grapl_common.debugger.vsc_debugger import wait_for_vsc_debugger
from grapl_common.env_helpers import SQSClientFactory
from grapl_common.grapl_logger import get_module_grapl_logger
from grapl_common.sqs.event_retriever import EventRetriever
from grapl_common.sqs.sqs_timeout_manager import (
    SqsTimeoutManager,
    SqsTimeoutManagerException,
)

wait_for_vsc_debugger(service="analyzer_executor")

LOGGER = get_module_grapl_logger()


async def _main() -> None:
    """
    Some TODOs to bring this inline with sqs-executor in Rust:
    - add the shortcut-to-DEAD_LETTER_QUEUE_URL case
    - pull the manual eventing out of `handle_events` and into an EventEmitter (maybe?)
    """
    queue_url = os.environ["SOURCE_QUEUE_URL"]
    retriever = EventRetriever(queue_url=queue_url)
    sqs_client = SQSClientFactory(boto3).from_env()
    analyzer_executor = AnalyzerExecutor.from_env()

    for sqs_message in retriever.retrieve():
        # We'd feed this coroutine into the timeout manager.
        message_handle_coroutine = analyzer_executor.handle_events(sqs_message.body, {})
        # While we're waiting for that future to complete, keep telling SQS
        # "hey, we're working on it" so it doesn't become visible on the
        # input queue again.
        timeout_manager = SqsTimeoutManager(
            sqs_client=sqs_client,
            queue_url=queue_url,
            receipt_handle=sqs_message.receipt_handle,
            message_id=sqs_message.message_id,
            coroutine=message_handle_coroutine,
        )
        try:
            await timeout_manager.keep_alive()
        except SqsTimeoutManagerException:
            LOGGER.error("SQS Timeout Manager exception", exc_info=True)


asyncio.run(_main())