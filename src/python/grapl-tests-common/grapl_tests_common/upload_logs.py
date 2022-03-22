from __future__ import annotations

import json
import os
import random
import string
import time
import uuid
from dataclasses import dataclass
from datetime import datetime
from os import PathLike
from typing import TYPE_CHECKING, Optional, cast

from grapl_common.env_helpers import S3ClientFactory, SQSClientFactory
from python_proto.pipeline import Metadata, OldEnvelope

if TYPE_CHECKING:
    from mypy_boto3_s3 import S3Client
    from mypy_boto3_sqs import SQSClient

import boto3
import zstd  # type: ignore


def rand_str(l: int) -> str:
    return "".join(
        random.choice(string.ascii_uppercase + string.digits) for _ in range(l)
    )


def into_sqs_message(bucket: str, key: str) -> str:
    return json.dumps(
        {
            "Records": [
                {
                    "awsRegion": "us-east-1",
                    "eventTime": datetime.utcnow().isoformat(),
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


@dataclass
class GeneratorOptions:
    bucket: str
    queue_url: str
    key_infix: str


class SysmonGeneratorOptions(GeneratorOptions):
    def __init__(self, bucket: str, queue_url: str) -> None:
        super().__init__(
            queue_url=queue_url,
            bucket=bucket,
            key_infix="sysmon",
        )


class OSQueryGeneratorOptions(GeneratorOptions):
    def __init__(self, bucket: str, queue_url: str) -> None:
        super().__init__(
            queue_url=queue_url,
            bucket=bucket,
            key_infix="osquery",
        )


def upload_logs(
    deployment_name: str,
    logfile: PathLike,
    generator_options: GeneratorOptions,
    s3_client: Optional[S3Client] = None,
    sqs_client: Optional[SQSClient] = None,
) -> None:
    print(f"Writing events from {logfile} to {deployment_name}")

    requires_manual_eventing = deployment_name == "local-grapl"

    s3 = s3_client or S3ClientFactory(boto3).from_env()
    sqs = sqs_client or SQSClientFactory(boto3).from_env()

    f = open(logfile, "rb")
    compressed_log_data = cast(bytes, zstd.compress(f.read()))
    original_filename = os.path.basename(f.name)
    compressed_file_name = original_filename + ".zstd"

    bucket = generator_options.bucket
    queue_url = generator_options.queue_url

    epoch = int(time.time())

    key = (
        str(epoch - (epoch % (24 * 60 * 60)))
        + f"/{generator_options.key_infix}/"
        + compressed_file_name
    )

    envelope = OldEnvelope(
        metadata=Metadata(
            tenant_id=uuid.uuid4(),  # FIXME: be smarter here.
            trace_id=uuid.uuid4(),  # FIXME: and here.
            event_source_id=uuid.uuid4(),  # FIXME: and here.
            created_time=datetime.utcnow(),
            last_updated_time=datetime.utcnow(),
        ),
        inner_message=compressed_log_data,
        inner_type="(╯°□°)╯︵ ┻━┻",
    )

    s3.put_object(Body=envelope.serialize(), Bucket=bucket, Key=key)

    # local-grapl relies on manual eventing
    if requires_manual_eventing:
        sqs.send_message(
            QueueUrl=queue_url,
            MessageBody=into_sqs_message(bucket=bucket, key=key),
        )

    print(f"Completed uploading {logfile} at {time.ctime()}")


def upload_sysmon_logs(
    deployment_name: str,
    logfile: PathLike,
    log_bucket: str,
    queue_url: str,
    s3_client: Optional[S3Client] = None,
    sqs_client: Optional[SQSClient] = None,
) -> None:

    upload_logs(
        deployment_name=deployment_name,
        logfile=logfile,
        generator_options=SysmonGeneratorOptions(
            bucket=log_bucket, queue_url=queue_url
        ),
        s3_client=s3_client,
        sqs_client=sqs_client,
    )


def upload_osquery_logs(
    deployment_name: str,
    logfile: PathLike,
    log_bucket: str,
    queue_url: str,
    s3_client: Optional[S3Client] = None,
    sqs_client: Optional[SQSClient] = None,
) -> None:
    upload_logs(
        deployment_name=deployment_name,
        logfile=logfile,
        generator_options=OSQueryGeneratorOptions(
            bucket=log_bucket, queue_url=queue_url
        ),
        s3_client=s3_client,
        sqs_client=sqs_client,
    )
