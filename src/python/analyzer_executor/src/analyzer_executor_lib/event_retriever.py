from __future__ import annotations

import json
import time
import traceback
from typing import TYPE_CHECKING, Iterator, List, cast

import boto3  # type: ignore
import botocore.exceptions  # type: ignore
from analyzer_executor_lib.grapl_logger import get_module_grapl_logger
from analyzer_executor_lib.s3_types import MessageWithReceipt
from grapl_common.env_helpers import SQSClientFactory

if TYPE_CHECKING:
    from mypy_boto3_sqs import SQSClient

LOGGER = get_module_grapl_logger()


def _ensure_alive(sqs: SQSClient) -> None:
    while True:
        try:
            if "QueueUrls" not in sqs.list_queues(
                QueueNamePrefix="grapl-analyzer-executor-queue"
            ):
                LOGGER.info("Waiting for grapl-analyzer-executor-queue to be created")
                time.sleep(2)
                continue
        except (
            botocore.exceptions.BotoCoreError,
            botocore.exceptions.ClientError,
            botocore.parsers.ResponseParserError,
        ):
            LOGGER.debug("Waiting for SQS to become available", exc_info=True)
            time.sleep(2)
            continue
        return


class EventRetriever:
    def __init__(
        self,
        queue_url: str,
    ) -> None:
        self.queue_url = queue_url

    def retrieve(self) -> Iterator[MessageWithReceipt]:
        """
        Yield batches of S3Put records from SQS.
        """
        while True:
            try:
                sqs = SQSClientFactory(boto3).from_env()
                _ensure_alive(sqs)

                res = sqs.receive_message(
                    QueueUrl=self.queue_url,
                    WaitTimeSeconds=3,
                    MaxNumberOfMessages=10,
                )

                messages = res.get("Messages", [])
                if not messages:
                    LOGGER.info("queue was empty")

                s3_events = cast(
                    List[MessageWithReceipt],
                    [
                        (json.loads(msg["Body"]), msg["ReceiptHandle"])
                        for msg in messages
                    ],
                )
                for body, receipt_handle in s3_events:
                    yield body, receipt_handle

                    sqs.delete_message(
                        QueueUrl=self.queue_url,
                        ReceiptHandle=receipt_handle,
                    )

            except Exception as e:
                LOGGER.error(traceback.format_exc())
                time.sleep(2)
