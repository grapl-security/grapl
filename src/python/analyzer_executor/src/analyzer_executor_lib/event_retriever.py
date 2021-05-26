from __future__ import annotations

import time
import traceback
from typing import TYPE_CHECKING, Iterator

import boto3
import botocore.exceptions
from analyzer_executor_lib.sqs_types import SQSMessage
from grapl_common.env_helpers import SQSClientFactory
from grapl_common.grapl_logger import get_module_grapl_logger

if TYPE_CHECKING:
    from mypy_boto3_sqs import SQSClient

LOGGER = get_module_grapl_logger()


def _ensure_alive(sqs: SQSClient, queue_url: str) -> None:
    queue_name = queue_url.split("/")[-1]
    while True:
        try:
            if "QueueUrls" not in sqs.list_queues(QueueNamePrefix=queue_name):
                LOGGER.info(f"Waiting for {queue_name} to be created")
                time.sleep(5)
                continue
        except (
            botocore.exceptions.BotoCoreError,
            botocore.exceptions.ClientError,
            botocore.parsers.ResponseParserError,
        ):
            LOGGER.debug("Waiting for SQS to become available", exc_info=True)
            time.sleep(5)
            continue
        return


class EventRetriever:
    def __init__(
        self,
        queue_url: str,
    ) -> None:
        self.queue_url = queue_url

    def retrieve(self) -> Iterator[SQSMessage]:
        """
        Yield batches of S3Put records from SQS.
        """
        while True:
            try:
                sqs = SQSClientFactory(boto3).from_env()
                _ensure_alive(sqs, self.queue_url)

                res = sqs.receive_message(
                    QueueUrl=self.queue_url,
                    WaitTimeSeconds=3,
                    MaxNumberOfMessages=10,
                )

                messages = res.get("Messages", [])
                if not messages:
                    LOGGER.info("queue was empty")

                s3_events = [SQSMessage(msg) for msg in messages]
                for sqs_message in s3_events:
                    yield sqs_message

                    sqs.delete_message(
                        QueueUrl=self.queue_url,
                        ReceiptHandle=sqs_message.receipt_handle,
                    )

            except Exception as e:
                LOGGER.error(traceback.format_exc())
                time.sleep(2)
