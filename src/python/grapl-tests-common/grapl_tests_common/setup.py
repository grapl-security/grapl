import logging
import subprocess
import sys
from os import environ
from sys import stdout
from typing import Any, NamedTuple, Sequence

import boto3  # type: ignore
import pytest
import requests
from grapl_tests_common.sleep import verbose_sleep
from grapl_tests_common.types import (
    AnalyzerUpload,
    S3ServiceResource,
    SqsServiceResource,
)
from grapl_tests_common.upload_test_data import UploadTestData
from grapl_tests_common.wait import WaitForS3Bucket, WaitForSqsQueue, wait_for

BUCKET_PREFIX = environ["BUCKET_PREFIX"]
assert BUCKET_PREFIX == "local-grapl"

logging.basicConfig(stream=stdout, level=logging.INFO)


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
    # You may want to sleep(30) to let the pipeline do its thing, but setup won't force it.


def _after_tests() -> None:
    """
    Add any "after tests are executed, but before docker-compose down" stuff here.
    """
    # Issue a command to dgraph to export the whole database.
    # This is then stored on a volume, `compose_artifacts`.
    # The contents of the volume are made available to Github Actions via `dump_compose_artifacts.py`.
    export_request = requests.get("http://grapl-master-graph-db:8080/admin/export")
    assert export_request.json()["code"] == "Success"


def exec_pytest() -> None:
    result = pytest.main(
        [
            "-s",  # disable stdout capture
        ]
    )
    _after_tests()

    sys.exit(result)
