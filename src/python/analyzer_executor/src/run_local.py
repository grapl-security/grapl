import json
import time
import traceback

import boto3  # type: ignore
import botocore.exceptions  # type: ignore
from analyzer_executor_lib.analyzer_executor import IS_LOCAL, LOGGER, lambda_handler_fn

if IS_LOCAL:
    while True:
        try:
            sqs = boto3.client(
                "sqs",
                region_name="us-east-1",
                endpoint_url="http://sqs.us-east-1.amazonaws.com:9324",
                aws_access_key_id="dummy_cred_aws_access_key_id",
                aws_secret_access_key="dummy_cred_aws_secret_access_key",
            )

            alive = False
            while not alive:
                try:
                    if "QueueUrls" not in sqs.list_queues(
                        QueueNamePrefix="grapl-analyzer-executor-queue"
                    ):
                        LOGGER.info(
                            "Waiting for grapl-analyzer-executor-queue to be created"
                        )
                        time.sleep(2)
                        continue
                except (
                    botocore.exceptions.BotoCoreError,
                    botocore.exceptions.ClientError,
                    botocore.parsers.ResponseParserError,
                ):
                    LOGGER.info("Waiting for SQS to become available")
                    time.sleep(2)
                    continue
                alive = True

            res = sqs.receive_message(
                QueueUrl="http://sqs.us-east-1.amazonaws.com:9324/queue/grapl-analyzer-executor-queue",
                WaitTimeSeconds=3,
                MaxNumberOfMessages=10,
            )

            messages = res.get("Messages", [])
            if not messages:
                LOGGER.warning("queue was empty")

            s3_events = [
                (json.loads(msg["Body"]), msg["ReceiptHandle"]) for msg in messages
            ]
            for s3_event, receipt_handle in s3_events:
                lambda_handler_fn(s3_event, {})

                sqs.delete_message(
                    QueueUrl="http://sqs.us-east-1.amazonaws.com:9324/queue/grapl-analyzer-executor-queue",
                    ReceiptHandle=receipt_handle,
                )

        except Exception as e:
            LOGGER.error(traceback.format_exc())
            time.sleep(2)
