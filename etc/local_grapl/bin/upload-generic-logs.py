#!/usr/bin/env python
"""
TODO: Move this into `graplctl upload`.
"""
import argparse
import json
import os
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


def main(deployment_name, logfile):
    print(f"Writing events to {deployment_name}")
    sqs = None
    # local-grapl deployment_name is reserved for running Grapl locally
    if deployment_name == "local-grapl":
        s3 = boto3.client(
            "s3",
            endpoint_url=os.environ["GRAPL_AWS_ENDPOINT"],
            aws_access_key_id=os.environ["GRAPL_AWS_ACCESS_KEY_ID"],
            aws_secret_access_key=os.environ["GRAPL_AWS_ACCESS_KEY_SECRET"],
        )
        sqs = boto3.client("sqs", endpoint_url=os.environ["GRAPL_AWS_ENDPOINT"])

    else:
        s3 = boto3.client("s3")

    with open(logfile, "rb") as b:
        body = json.load(b)

        body = [line for line in body]

    def chunker(seq, size):
        return [seq[pos : pos + size] for pos in range(0, len(seq), size)]

    for chunks in chunker(body, 50):
        c_body = zstd.compress(json.dumps(chunks), 4)
        epoch = int(time.time())

        key = (
            str(epoch - (epoch % (24 * 60 * 60)))
            + "/injected/"
            + str(epoch)
            + rand_str(3)
        )

        s3.put_object(
            Body=c_body, Bucket="{}-raw-log-bucket".format(deployment_name), Key=key
        )
        # local-grapl relies on manual eventing
        if sqs:
            sqs.send_message(
                QueueUrl=f"{os.environ['GRAPL_AWS_ENDPOINT']}/queue/grapl-generic-graph-generator-queue",
                MessageBody=into_sqs_message(
                    bucket="{}-sysmon-log-bucket".format(deployment_name), key=key
                ),
            )
    print(time.ctime())


def parse_args():
    parser = argparse.ArgumentParser(description="Send generic logs to Grapl")
    parser.add_argument("--deployment_name", dest="deployment_name", required=True)
    parser.add_argument("--logfile", dest="logfile", required=True)
    return parser.parse_args()


if __name__ == "__main__":

    args = parse_args()
    main(args.deployment_name, args.logfile)
