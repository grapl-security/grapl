import boto3
import subprocess
from os import environ
from typing import Any
import logging
from resources import wait_on_resources
from sys import stdout

# mypy later maybe
S3ServiceResource = Any

BUCKET_PREFIX = environ["BUCKET_PREFIX"]
# assert BUCKET_PREFIX == "local-grapl"


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
    """
    i hate this lol
    """
    logging.info(f"Running upload-sysmon-logs")

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


def main() -> None:
    logging.basicConfig(stream=stdout, level=logging.INFO)

    s3_client = _create_s3_client()
    wait_on_resources(s3_client, BUCKET_PREFIX)
    _upload_analyzers(s3_client)
    _upload_test_data(s3_client)


if __name__ == "__main__":
    main()
