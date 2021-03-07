from typing import Optional

import pulumi_aws as aws
from infra import util
from infra.util import IS_LOCAL

import pulumi


class UI(pulumi.ComponentResource):
    """ Represents the web GUI for Grapl."""

    def __init__(
        self, name: str, opts: Optional[pulumi.ResourceOptions] = None
    ) -> None:
        super().__init__("grapl:UI", name, None, opts)

        logical_bucket_name = "engagement-ux-bucket"
        physical_bucket_name = f"{name}-{logical_bucket_name}"

        # It appears that the website configuration is not available
        # in MinIO, which we currently use for s3 in local grapl. When
        # interacting with local grapl, we'll just leave it out.
        aws_only_args = (
            {
                "website": aws.s3.BucketWebsiteArgs(
                    index_document="index.html",
                )
            }
            if not IS_LOCAL
            else {}
        )

        self.bucket = aws.s3.Bucket(
            logical_bucket_name,
            bucket=physical_bucket_name,
            tags={"grapl deployment": pulumi.get_stack()},
            opts=util.import_aware_opts(physical_bucket_name, parent=self),
            **aws_only_args,
        )

        self.register_outputs({})
