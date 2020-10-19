from os import environ
from grapl_tests_common.wait import wait_for, WaitForS3Bucket, WaitForSqsQueue
from grapl_tests_common.sleep import verbose_sleep
from grapl_tests_common.types import (
    S3ServiceResource,
    SqsServiceResource,
    AnalyzerUpload,
)
from grapl_tests_common.upload_test_data import UploadTestData
from sys import stdout
from typing import Any, NamedTuple, Sequence
import boto3  # type: ignore
import logging
import pytest
import subprocess
import sys

BUCKET_PREFIX = environ["BUCKET_PREFIX"]
assert BUCKET_PREFIX == "local-grapl"


def _upload_analyzers(
    s3_client: S3ServiceResource, analyzers: Sequence[AnalyzerUpload]
) -> None:
    """
    Basically reimplementing upload_local_analyzers.sh
    Janky, since Jesse will have an analyzer-uploader service pretty soon.
    """

    bucket = f"{BUCKET_PREFIX}-analyzers-bucket"
    for (local_path, s3_key) in analyzers:
        logging.info(f"S3 uploading analyzer from {local_path}")
        with open(local_path, "r") as f:
            s3_client.put_object(Body=f.read(), Bucket=bucket, Key=s3_key)


def _upload_test_data(
    s3_client: S3ServiceResource, test_data: Sequence[UploadTestData]
) -> None:
    logging.info(f"Uploading test data...")

    for datum in test_data:
        datum.upload(s3_client)


def _create_s3_client() -> S3ServiceResource:
    return boto3.client(
        "s3",
        endpoint_url="http://s3:9000",
        aws_access_key_id="minioadmin",
        aws_secret_access_key="minioadmin",
    )


def _create_sqs_client() -> SqsServiceResource:
    # mostly cribbed from upload-sysmon-logs
    return boto3.client(
        "sqs",
        endpoint_url="http://sqs:9324",
        region_name="us-east-1",
        aws_access_key_id="minioadmin",
        aws_secret_access_key="minioadmin",
    )


def setup(
    analyzers: Sequence[AnalyzerUpload],
    test_data: Sequence[UploadTestData],
) -> None:
    logging.basicConfig(stream=stdout, level=logging.INFO)
    verbose_sleep(10, "awaiting local aws")

    s3_client = _create_s3_client()
    sqs_client = _create_sqs_client()

    wait_for(
        [
            # for uploading analyzers
            WaitForS3Bucket(s3_client, f"{BUCKET_PREFIX}-analyzers-bucket"),
            # for upload-sysmon-logs.py
            WaitForS3Bucket(s3_client, f"{BUCKET_PREFIX}-sysmon-log-bucket"),
            WaitForSqsQueue(sqs_client, "grapl-sysmon-graph-generator-queue"),
        ]
    )

    _upload_analyzers(s3_client, analyzers)
    _upload_test_data(s3_client, test_data)

    verbose_sleep(30, "let the pipeline do its thing")


def exec_pytest() -> None:
    result = pytest.main(
        [
            "-s",  # disable stdout capture
        ]
    )
    sys.exit(result)
