from typing import Optional

import pulumi_aws as aws
from infra import util
from infra.util import DEPLOYMENT_NAME, IS_LOCAL

import pulumi


class EngagementUX(pulumi.ComponentResource):
    """ Represents the web GUI for Grapl."""

    def __init__(self, opts: Optional[pulumi.ResourceOptions] = None) -> None:
        super().__init__("grapl:UI", DEPLOYMENT_NAME, None, opts)

        logical_bucket_name = "engagement-ux-bucket"
        physical_bucket_name = f"{DEPLOYMENT_NAME}-{logical_bucket_name}"

        # It appears that the website configuration is not available
        # in MinIO, which we currently use for s3 in local grapl. When
        # interacting with local grapl, we'll just leave it out.
        website_args = (
            aws.s3.BucketWebsiteArgs(
                index_document="index.html",
            )
            if not IS_LOCAL
            else None
        )

        self.bucket = aws.s3.Bucket(
            logical_bucket_name,
            bucket=physical_bucket_name,
            website=website_args,
            opts=util.import_aware_opts(physical_bucket_name, parent=self),
        )

        self.register_outputs({})
