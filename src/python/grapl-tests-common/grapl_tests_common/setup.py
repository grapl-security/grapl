from __future__ import annotations

import logging
import subprocess
import sys
from os import environ
from sys import stdout
from typing import TYPE_CHECKING, Any, NamedTuple, Sequence

import boto3  # type: ignore
import pytest
import requests
from grapl_common.env_helpers import S3ClientFactory, SQSClientFactory
from grapl_tests_common.dump_dynamodb import dump_dynamodb
from grapl_tests_common.sleep import verbose_sleep
from grapl_tests_common.types import (
    AnalyzerUpload,
    S3ServiceResource,
    SqsServiceResource,
)
from grapl_tests_common.upload_test_data import UploadTestData
from grapl_tests_common.wait import WaitForS3Bucket, WaitForSqsQueue, wait_for

if TYPE_CHECKING:
    from mypy_boto3_s3 import S3Client
    from mypy_boto3_sqs import SQSClient

DEPLOYMENT_NAME = environ["DEPLOYMENT_NAME"]
assert DEPLOYMENT_NAME == "local-grapl"

# Toggle if you want to dump databases, logs, etc.
DUMP_ARTIFACTS = bool(environ.get("DUMP_ARTIFACTS", False))

logging.basicConfig(stream=stdout, level=logging.INFO)


def _upload_analyzers(
    s3_client: S3ServiceResource, analyzers: Sequence[AnalyzerUpload]
) -> None:
    """
    Basically reimplementing upload_local_analyzers.sh
    Janky, since Jesse will have an analyzer-uploader service pretty soon.
    """

    bucket = f"{DEPLOYMENT_NAME}-analyzers-bucket"
    for (local_path, s3_key) in analyzers:
        logging.info(f"S3 uploading analyzer from {local_path}")
        with open(local_path, "r") as f:
            s3_client.put_object(Body=f.read(), Bucket=bucket, Key=s3_key)


def _upload_test_data(
    s3_client: S3ServiceResource,
    sqs_client: SQSClient,
    test_data: Sequence[UploadTestData],
) -> None:
    logging.info(f"Uploading test data...")

    for datum in test_data:
        datum.upload(s3_client, sqs_client)


def _create_s3_client() -> S3Client:
    return S3ClientFactory(boto3).from_env()


def _create_sqs_client() -> SQSClient:
    return SQSClientFactory(boto3).from_env()


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
            WaitForS3Bucket(s3_client, f"{DEPLOYMENT_NAME}-analyzers-bucket"),
            # for upload-sysmon-logs.py
            WaitForS3Bucket(s3_client, f"{DEPLOYMENT_NAME}-sysmon-log-bucket"),
            WaitForSqsQueue(sqs_client, f"{DEPLOYMENT_NAME}-sysmon-generator-queue"),
        ]
    )

    _upload_analyzers(s3_client, analyzers)
    _upload_test_data(s3_client, sqs_client, test_data)
    # You may want to sleep(30) to let the pipeline do its thing, but setup won't force it.


def _after_tests() -> None:
    """
    Add any "after tests are executed, but before docker-compose down" stuff here.
    """

    # Issue a command to dgraph to export the whole database.
    # This is then stored on a volume, `dgraph_export` (defined in docker-compose.yml)
    # The contents of the volume are made available to Github Actions via `dump_artifacts.py`.
    if DUMP_ARTIFACTS:
        logging.info("Executing post-test database dumps")
        export_request = requests.get("http://dgraph.grapl.test:8080/admin/export")
        assert export_request.json()["code"] == "Success"
        dump_dynamodb()


def exec_pytest() -> None:
    result = pytest.main(
        [
            "-s",
        ]
    )  # disable stdout capture
    _after_tests()

    sys.exit(result)
