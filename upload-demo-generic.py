#!/usr/bin/env python
import json

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
    return ''.join(random.choice(string.ascii_uppercase + string.digits)
                   for _ in range(l))


def main(prefix):

    s3 = boto3.client('s3')

    with open('./injected_logs.json', 'rb') as b:
        body = json.load(b)

        body = [line for line in body]

    def chunker(seq, size):
        return [seq[pos:pos + size] for pos in range(0, len(seq), size)]

    for chunks in chunker(body, 50):
        c_body = zstd.compress(json.dumps(chunks), 4)
        epoch = int(time.time())

        s3.put_object(
            Body=c_body,
            Bucket="{}-raw-log-bucket".format(prefix),
            Key=str(epoch - (epoch % (24 * 60 * 60))) + "/injected/" +
                str(epoch) + rand_str(3)
        )

    print(time.ctime())

if __name__ == '__main__':

    if len(sys.argv) != 2:
        raise Exception("Provide bucket prefix as first argument")
    else:
        main(sys.argv[1])
