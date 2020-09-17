#!/usr/bin/env python
"""
Despite the path, this is *not* tied just to Local Grapl, and can also be used on true S3 buckets.
"""

import argparse
import json
import random
import string
import time
from datetime import datetime

import boto3

import zstd


def rand_str(l):
    # type: (int) -> str
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


def main(prefix, logfile, delay, batch_size):
    print(
        f"Writing events to {prefix} with {delay} seconds between batches of {batch_size}"
    )
    sqs = None
    # local-grapl prefix is reserved for running Grapl locally
    if prefix == "local-grapl":
        s3 = boto3.client(
            "s3",
            endpoint_url="http://localhost:9000",
            aws_access_key_id="minioadmin",
            aws_secret_access_key="minioadmin",
            region_name="us-east-3",
        )
        sqs = boto3.client(
            "sqs",
            endpoint_url="http://localhost:9324",
            region_name="us-east-1",
            aws_access_key_id="dummy_cred_aws_access_key_id",
            aws_secret_access_key="dummy_cred_aws_secret_access_key",
        )

    else:
        s3 = boto3.client("s3")

    with open(logfile, "rb") as b:
        body = b.readlines()
        body = [line for line in body]

    def chunker(seq, size):
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
            sqs.send_message(
                QueueUrl="http://localhost:9324/queue/grapl-sysmon-graph-generator-queue",
                MessageBody=into_sqs_message(
                    bucket="{}-sysmon-log-bucket".format(prefix), key=key
                ),
            )

        time.sleep(delay)

    print(f"Completed uploading at {time.ctime()}")


def parse_args():
    parser = argparse.ArgumentParser(description="Send sysmon logs to Grapl")
    parser.add_argument("--bucket_prefix", dest="bucket_prefix", required=True)
    parser.add_argument("--logfile", dest="logfile", required=True, help="ie $GRAPLROOT/etc/sample_data/eventlog.xml")
    parser.add_argument("--delay", dest="delay", default=0, type=int)
    parser.add_argument("--batch-size", dest="batch_size", default=100, type=int)
    return parser.parse_args()


if __name__ == "__main__":

    args = parse_args()
    if args.bucket_prefix is None:
        raise Exception("Provide bucket prefix as first argument")
    else:
        main(args.bucket_prefix, args.logfile, args.delay, args.batch_size)
