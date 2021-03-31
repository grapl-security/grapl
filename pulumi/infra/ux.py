from typing import Optional

import pulumi_aws as aws
from infra.bucket import Bucket
from infra.config import DEPLOYMENT_NAME

import pulumi


class EngagementUX(pulumi.ComponentResource):
    """ Represents the web GUI for Grapl."""

    def __init__(self, opts: Optional[pulumi.ResourceOptions] = None) -> None:
        super().__init__("grapl:EngagementUX", DEPLOYMENT_NAME, None, opts)

        self.bucket = Bucket(
            "engagement-ux-bucket",
            website_args=aws.s3.BucketWebsiteArgs(
                index_document="index.html",
            ),
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.register_outputs({})
