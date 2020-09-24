from os import environ
import resources
from sys import stdout
from typing import Any
import boto3
import logging
import pytest
import subprocess

# mypy later maybe
S3ServiceResource = Any
SqsServiceResource = Any

BUCKET_PREFIX = environ["BUCKET_PREFIX"]
assert BUCKET_PREFIX == "local-grapl"


def _upload_analyzers(s3_client: S3ServiceResource) -> None:
    """
    Basically reimplementing upload_local_analyzers.sh
    Janky, since Jesse will have an analyzer-uploader service pretty soon.
    """
    to_upload = [
        (
            "/home/grapl/etc/local_grapl/suspicious_svchost/main.py",
            "analyzers/suspicious_svchost/main.py",
        ),
        (
            "/home/grapl/etc/local_grapl/unique_cmd_parent/main.py",
            "analyzers/unique_cmd_parent/main.py",
        ),
    ]
    bucket = f"{BUCKET_PREFIX}-analyzers-bucket"
    for (local_path, s3_key) in to_upload:
        logging.info(f"S3 uploading {local_path}")
        with open(local_path, "r") as f:
            s3_client.put_object(Body=f.read(), Bucket=bucket, Key=s3_key)


def _upload_test_data(s3_client: S3ServiceResource) -> None:
    logging.info(f"Running upload-sysmon-logs")

    # i hate this lol
    # but it's probably better than mucking with path and importing that module...
    subprocess.run(
        [
            "python3",
            "/home/grapl/etc/local_grapl/bin/upload-sysmon-logs.py",
            "--bucket_prefix",
            BUCKET_PREFIX,
            "--logfile",
            "/home/grapl/etc/sample_data/eventlog.xml",
            "--use-links",
            "True",
        ]
    )


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


def main() -> None:
    logging.basicConfig(stream=stdout, level=logging.INFO)

    s3_client = _create_s3_client()
    sqs_client = _create_sqs_client()

    wait_for = [
        # for uploading analyzers
        resources.WaitForS3Bucket(s3_client, f"{BUCKET_PREFIX}-analyzers-bucket"),
        # for upload-sysmon-logs.py
        resources.WaitForS3Bucket(s3_client, f"{BUCKET_PREFIX}-sysmon-log-bucket"),
        resources.WaitForSqsQueue(sqs_client, "grapl-sysmon-graph-generator-queue"),
    ]
    resources.wait_on_resources(wait_for)

    _upload_analyzers(s3_client)
    _upload_test_data(s3_client)

    import tests

    return pytest.main()


if __name__ == "__main__":
    main()
