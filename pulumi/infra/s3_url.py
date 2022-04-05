from typing import Mapping

import pulumi_aws as aws
from infra import config

import pulumi


def get_s3_url(obj: aws.s3.BucketObject) -> pulumi.Output[str]:
    def _inner(inputs: Mapping[str, str]) -> str:
        if config.LOCAL_GRAPL:
            return f"http://{config.HOST_IP_IN_NOMAD}:4566/{inputs['bucket']}/{inputs['key']}"
        return f"https://{inputs['bucket']}.s3.amazonaws.com/{inputs['key']}"

    return pulumi.Output.all(bucket=obj.bucket, key=obj.key).apply(_inner)
