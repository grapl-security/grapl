import argparse
import json
import random
import string
import time
from typing import List
from datetime import datetime

import boto3  # type: ignore

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


def upload_sysmon_logs(
    prefix: str,
    logfile: str,
    delay: int = 0,
    batch_size: int = 100,
    use_links: bool = False,
) -> None:
    """
    `use_links` meaning use "s3", "sqs" as opposed to localhost
    """
    print(
        f"Writing events to {prefix} with {delay} seconds between batches of {batch_size}"
    )
    sqs = None
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
            endpoint_url="http://sqs:9324" if use_links else "http://localhost:9324",
            region_name="us-east-1",
            aws_access_key_id="dummy_cred_aws_access_key_id",
            aws_secret_access_key="dummy_cred_aws_secret_access_key",
        )

    else:
        s3 = boto3.client("s3")

    with open(logfile, "rb") as b:
        body = b.readlines()
        body = [line for line in body]

    def chunker(seq: List[bytes], size: int) -> List[List[bytes]]:
        return [seq[pos : pos + size] for pos in range(0, len(seq), size)]

    for chunks in chunker(body, batch_size):
        c_body = zstd.compress(b"\n".join(chunks).replace(b"\n\n", b"\n"), 4)
        epoch = int(time.time())

        key = (
            str(epoch - (epoch % (24 * 60 * 60)))
            + "/sysmon/"
            + str(epoch)
            + rand_str(3)
        )
        s3.put_object(
            Body=c_body, Bucket="{}-sysmon-log-bucket".format(prefix), Key=key
        )

        # local-grapl relies on manual eventing
        if sqs:
            endpoint_url = (
                "http://sqs:9324" if use_links else "http://localhost:9324",
            )
            sqs.send_message(
                QueueUrl=f"{endpoint_url}/queue/grapl-sysmon-graph-generator-queue",
                MessageBody=into_sqs_message(
                    bucket="{}-sysmon-log-bucket".format(prefix), key=key
                ),
            )

        time.sleep(delay)

    print(f"Completed uploading at {time.ctime()}")
