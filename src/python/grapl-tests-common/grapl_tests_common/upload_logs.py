from dataclasses import dataclass
from datetime import datetime
from typing import Callable, List, Iterator, Optional, cast
from sys import maxsize
import boto3  # type: ignore
import json
import random
import string
import time
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
    bucket_suffix: str
    queue_name: str
    key_infix: str

    def encode_chunk(self, input: List[bytes]) -> bytes:
        raise NotImplementedError()


class SysmonGeneratorOptions(GeneratorOptions):
    def __init__(self) -> None:
        super().__init__(
            queue_name="grapl-sysmon-graph-generator-queue",
            bucket_suffix="sysmon-log-bucket",
            key_infix="sysmon",
        )

    def encode_chunk(self, input: List[bytes]) -> bytes:
        # zstd encoded line delineated xml
        return cast(bytes, zstd.compress(b"\n".join(input).replace(b"\n\n", b"\n"), 4))


def upload_logs(
    prefix: str,
    logfile: str,
    generator_options: GeneratorOptions,
    delay: int = 0,
    batch_size: Optional[int] = 100,
    use_links: bool = False,
) -> None:
    """
    `use_links` meaning use "s3", "sqs" as opposed to localhost
    set `batch_size` to None to disable batching
    """
    print(
        f"Writing events to {prefix} with {delay} seconds between batches of {batch_size}"
    )

    # Ugly hack to cheaply disable batching
    batch_size = batch_size if batch_size is not None else maxsize

    sqs = None
    local_sqs_endpoint_url = "http://sqs:9324" if use_links else "http://localhost:9324"
    # local-grapl prefix is reserved for running Grapl locally
    if prefix == "local-grapl":
        s3 = boto3.client(
            "s3",
            endpoint_url="http://s3:9000" if use_links else "http://localhost:9000",
            aws_access_key_id="minioadmin",
            aws_secret_access_key="minioadmin",
            region_name="us-east-3",
        )
        sqs = boto3.client(
            "sqs",
            endpoint_url=local_sqs_endpoint_url,
            region_name="us-east-1",
            aws_access_key_id="dummy_cred_aws_access_key_id",
            aws_secret_access_key="dummy_cred_aws_secret_access_key",
        )

    else:
        s3 = boto3.client("s3")

    with open(logfile, "rb") as b:
        body = b.readlines()
        body = [line for line in body]

    def chunker(seq: List[bytes], size: int) -> Iterator[List[bytes]]:
        return (seq[pos : pos + size] for pos in range(0, len(seq), size))

    bucket = f"{prefix}-{generator_options.bucket_suffix}"

    for chunk in chunker(body, batch_size):
        chunk_body = generator_options.encode_chunk(chunk)
        epoch = int(time.time())

        key = (
            str(epoch - (epoch % (24 * 60 * 60)))
            + f"/{generator_options.key_infix}/"
            + str(epoch)
            + rand_str(3)
        )
        s3.put_object(Body=chunk_body, Bucket=bucket, Key=key)

        # local-grapl relies on manual eventing
        if sqs:
            sqs.send_message(
                QueueUrl=f"{local_sqs_endpoint_url}/queue/{generator_options.queue_name}",
                MessageBody=into_sqs_message(bucket=bucket, key=key),
            )

        time.sleep(delay)

    print(f"Completed uploading at {time.ctime()}")


def upload_sysmon_logs(
    prefix: str,
    logfile: str,
    delay: int = 0,
    batch_size: int = 100,
    use_links: bool = False,
) -> None:

    upload_logs(
        prefix=prefix,
        logfile=logfile,
        generator_options=SysmonGeneratorOptions(),
        delay=delay,
        batch_size=batch_size,
        use_links=use_links,
    )
