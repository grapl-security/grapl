#!/usr/bin/env python
"""
TODO: I believe this script is superceded by `upload-sysmon-logs`, which takes in both
--bucket-prefix and --log-file.
"""

import argparse
import json
from datetime import datetime

try:
    from typing import Any, Dict, Union, Optional
except:
    pass

import time
import string
import boto3
import random
import zstd
import sys


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


def main(prefix):

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
            Bucket="{}-sysmon-log-bucket".format(prefix),
            Key=str(epoch - (epoch % (24 * 60 * 60)))
            + "/sysmon/"
            + str(epoch)
            + rand_str(3),
        )
    print(time.ctime())


def parse_args():
    parser = argparse.ArgumentParser(description="Send demo logs to Grapl")
    parser.add_argument("--bucket_prefix", dest="bucket_prefix", required=True)
    return parser.parse_args()


if __name__ == "__main__":

    args = parse_args()
    if args.bucket_prefix is None:
        raise Exception("Provide bucket prefix as first argument")
    else:
        main(args.bucket_prefix)
