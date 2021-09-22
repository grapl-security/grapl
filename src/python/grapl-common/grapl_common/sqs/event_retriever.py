from __future__ import annotations

import time
import traceback
from typing import Iterator

import boto3
from grapl_common.sqs.sqs_types import SQSMessage
from grapl_common.env_helpers import SQSClientFactory
from grapl_common.grapl_logger import get_module_grapl_logger

LOGGER = get_module_grapl_logger()


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
