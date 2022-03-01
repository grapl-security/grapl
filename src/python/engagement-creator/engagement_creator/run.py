import asyncio
import os

import boto3
from engagement_creator.lib.engagement_creator import lambda_handler
from grapl_common.env_helpers import SQSClientFactory
from grapl_common.grapl_logger import get_module_grapl_logger
from grapl_common.grapl_tracer import get_tracer
from grapl_common.sqs.event_retriever import EventRetriever
from grapl_common.sqs.sqs_timeout_manager import (
    SqsTimeoutManager,
    SqsTimeoutManagerException,
)

LOGGER = get_module_grapl_logger()
TRACER = get_tracer(service_name="engagement_creator", module_name=__name__)


async def _main() -> None:
    """
    Some TODOs to bring this inline with sqs-executor in Rust:
    - add the shortcut-to-DEAD_LETTER_QUEUE_URL case
    """
    with TRACER.start_as_current_span("engagement_creator"):
        queue_url = os.environ["SOURCE_QUEUE_URL"]
        retriever = EventRetriever(queue_url=queue_url)
        sqs_client = SQSClientFactory(boto3).from_env()

        for sqs_message in retriever.retrieve():
            # We'd feed this coroutine into the timeout manager.
            message_handle_coroutine = lambda_handler(sqs_message.body, {})
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


def run() -> None:
    asyncio.run(_main())


if __name__ == "__main__":
    run()
