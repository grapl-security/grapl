import argparse
import json
import os
from datetime import datetime
from typing import Iterator

import boto3
from mypy_boto3_s3.client import S3Client
from mypy_boto3_sqs.client import SQSClient

IS_LOCAL = bool(os.environ.get("IS_LOCAL", False))


def into_sqs_message(bucket: str, key: str, region: str) -> str:
    return json.dumps(
        {
            "Records": [
                {
                    "eventTime": datetime.utcnow().isoformat(),
                    "awsRegion": region,
                    "principalId": {
                        "principalId": None,
                    },
                    "requestParameters": {
                        "sourceIpAddress": None,
                    },
                    "responseElements": {},
                    "s3": {
                        "schemaVersion": None,
                        "configurationId": None,
                        "bucket": {
                            "name": bucket,
                            "ownerIdentity": {
                                "principalId": None,
                            },
                        },
                        "object": {
                            "key": key,
                            "size": 0,
                            "urlDecodedKey": None,
                            "versionId": None,
                            "eTag": None,
                            "sequencer": None,
                        },
                    },
                }
            ]
        }
    )


def send_s3_event(
    sqs_client: SQSClient,
    queue_url: str,
    output_bucket: str,
    output_path: str,
):
    sqs_client.send_message(
        QueueUrl=queue_url,
        MessageBody=into_sqs_message(
            bucket=output_bucket,
            key=output_path,
            region=sqs_client.meta.region_name,
        ),
    )


def list_objects(client: S3Client, bucket: str) -> Iterator[str]:
    for page in client.get_paginator("list_objects_v2").paginate(Bucket=bucket):
        for entry in page["Contents"]:
            yield entry["Key"]


def get_sqs_client() -> SQSClient:
    if IS_LOCAL:
        return boto3.client(
            "sqs",
            endpoint_url=os.environ["AWS_ENDPOINT"],
            region_name=os.environ["AWS_REGION"],
            aws_access_key_id=os.environ["AWS_ACCESS_KEY_ID"],
            aws_secret_access_key=os.environ["AWS_ACCESS_KEY_SECRET"],
        )
    else:
        return boto3.client("sqs")


def get_s3_client() -> S3Client:
    if IS_LOCAL:
        return boto3.client(
            "s3",
            endpoint_url=os.environ["AWS_ENDPOINT"],
            aws_access_key_id=os.environ["AWS_ACCESS_KEY_ID"],
            aws_secret_access_key=os.environ["AWS_ACCESS_KEY_SECRET"],
        )

    else:
        return boto3.client("s3")


def main(deployment_name: str) -> None:
    s3, sqs = get_s3_client(), get_sqs_client()
    queue_name = deployment_name + "-graph-merger-queue"
    queue_url = sqs.get_queue_url(QueueName=queue_name)["QueueUrl"]

    bucket = deployment_name + "-subgraphs-generated-bucket"
    for key in list_objects(s3, bucket):
        send_s3_event(
            sqs,
            queue_url,
            bucket,
            key,
        )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Replay graph-merger events")
    parser.add_argument("--deployment_name", dest="deployment_name", required=True)
    return parser.parse_args()


if __name__ == "__main__":

    args = parse_args()
    if args.deployment_name is None:
        raise Exception("Provide deployment name as first argument")
    else:
        if args.deployment_name == "local-grapl":
            IS_LOCAL = True
        main(args.deployment_name)
