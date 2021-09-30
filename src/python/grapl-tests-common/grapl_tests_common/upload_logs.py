from __future__ import annotations

import json
import random
import string
import time
from dataclasses import dataclass
from datetime import datetime
from os import PathLike
from sys import maxsize
from typing import TYPE_CHECKING, Iterator, List, Optional, cast

from grapl_common.env_helpers import S3ClientFactory, SQSClientFactory

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

    def encode_chunk(self, input: List[bytes]) -> bytes:
        raise NotImplementedError()


class SysmonGeneratorOptions(GeneratorOptions):
    def __init__(self, bucket: str, queue_url: str) -> None:
        super().__init__(
            queue_url=queue_url,
            bucket=bucket,
            key_infix="sysmon",
        )

    def encode_chunk(self, input: List[bytes]) -> bytes:
        # zstd encoded line delineated xml
        return cast(bytes, zstd.compress(b"\n".join(input).replace(b"\n\n", b"\n"), 4))


class OSQueryGeneratorOptions(GeneratorOptions):
    def __init__(self, bucket: str, queue_url: str) -> None:
        super().__init__(
            queue_url=queue_url,
            bucket=bucket,
            key_infix="osquery",
        )

    def encode_chunk(self, input: List[bytes]) -> bytes:
        # zstd encoded line delineated xml
        return cast(bytes, zstd.compress(b"\n".join(input).replace(b"\n\n", b"\n"), 4))


def upload_logs(
    deployment_name: str,
    logfile: PathLike,
    generator_options: GeneratorOptions,
    delay: int = 0,
    batch_size: Optional[int] = 100,
    s3_client: Optional[S3Client] = None,
    sqs_client: Optional[SQSClient] = None,
) -> None:
    """
    set `batch_size` to None to disable batching
    """
    print(
        f"Writing events to {deployment_name} with {delay} seconds between batches of {batch_size}"
    )

    # Ugly hack to cheaply disable batching
    batch_size = batch_size if batch_size is not None else maxsize
    requires_manual_eventing = deployment_name == "local-grapl"
    s3 = s3_client or S3ClientFactory(boto3).from_env()
    sqs = sqs_client or SQSClientFactory(boto3).from_env()

    with open(logfile, "rb") as b:
        body = b.readlines()
        body = [line for line in body]

    def chunker(seq: List[bytes], size: int) -> Iterator[List[bytes]]:
        return (seq[pos : pos + size] for pos in range(0, len(seq), size))

    bucket = generator_options.bucket
    queue_url = generator_options.queue_url

    chunk_count = 0
    for chunk in chunker(body, batch_size):
        chunk_count += 1
        chunk_body = generator_options.encode_chunk(chunk)
        epoch = int(time.time())

        key = (
            str(epoch - (epoch % (24 * 60 * 60)))
            + f"/{generator_options.key_infix}/"
            + str(epoch)
            + rand_str(6)
        )
        s3.put_object(Body=chunk_body, Bucket=bucket, Key=key)

        # local-grapl relies on manual eventing
        if requires_manual_eventing:
            sqs.send_message(
                QueueUrl=queue_url,
                MessageBody=into_sqs_message(bucket=bucket, key=key),
            )

        time.sleep(delay)

    print(f"Completed uploading {chunk_count} chunks at {time.ctime()}")


def upload_sysmon_logs(
    deployment_name: str,
    logfile: PathLike,
    log_bucket: str,
    queue_url: str,
    delay: int = 0,
    batch_size: int = 100,
    s3_client: Optional[S3Client] = None,
    sqs_client: Optional[SQSClient] = None,
) -> None:

    upload_logs(
        deployment_name=deployment_name,
        logfile=logfile,
        generator_options=SysmonGeneratorOptions(
            bucket=log_bucket, queue_url=queue_url
        ),
        delay=delay,
        batch_size=batch_size,
        s3_client=s3_client,
        sqs_client=sqs_client,
    )


def upload_osquery_logs(
    deployment_name: str,
    logfile: PathLike,
    log_bucket: str,
    queue_url: str,
    delay: int = 0,
    batch_size: int = 100,
    s3_client: Optional[S3Client] = None,
    sqs_client: Optional[SQSClient] = None,
) -> None:
    upload_logs(
        deployment_name=deployment_name,
        logfile=logfile,
        generator_options=OSQueryGeneratorOptions(
            bucket=log_bucket, queue_url=queue_url
        ),
        delay=delay,
        batch_size=batch_size,
        s3_client=s3_client,
        sqs_client=sqs_client,
    )
