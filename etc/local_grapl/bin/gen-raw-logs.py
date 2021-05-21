#!/usr/bin/env python
"""
TODO: I believe this script is superceded by `upload-sysmon-logs`, which takes in both
--deployment-name and --log-file.
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


def main(deployment_name):

    s3 = boto3.client("s3")

    with open("./eventlog.xml", "rb") as b:
        body = b.readlines()
        body = [line for line in body]

    def chunker(seq, size):
        return [seq[pos : pos + size] for pos in range(0, len(seq), size)]

    for chunks in chunker(body, 50):
        c_body = zstd.compress(b"\n".join(chunks), 4)
        epoch = int(time.time())

        s3.put_object(
            Body=c_body,
            Bucket="{}-sysmon-log-bucket".format(deployment_name),
            Key=str(epoch - (epoch % (24 * 60 * 60)))
            + "/sysmon/"
            + str(epoch)
            + rand_str(3),
        )
    print(time.ctime())


def parse_args():
    parser = argparse.ArgumentParser(description="Send demo logs to Grapl")
    parser.add_argument("--deployment_name", dest="deployment_name", required=True)
    return parser.parse_args()


if __name__ == "__main__":

    args = parse_args()
    if args.deployment_name is None:
        raise Exception("Provide deployment name as first argument")
    else:
        main(args.deployment_name)
