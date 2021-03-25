from typing import Optional

import pulumi_aws as aws
from infra.config import DEPLOYMENT_NAME, import_aware_opts

import pulumi


class Bucket(aws.s3.Bucket):
    def __init__(
        self,
        logical_bucket_name: str,
        sse: bool = False,
        website_args: Optional[aws.s3.BucketWebsiteArgs] = None,
        parent: Optional[pulumi.Resource] = None,
    ) -> None:
        """Abstracts logic for creating an S3 bucket for our purposes.

        logical_bucket_name: What we call this bucket in Pulumi terms.

        sse: Whether or not to apply server-side encryption of
        bucket contents

        website_args: configuration for setting the bucket up to serve web
        content.

        parent: for use in ComponentResources; the Pulumi resource
        that "owns" this resource.

        """
        physical_bucket_name = bucket_physical_name(logical_bucket_name)

        sse_config = sse_configuration() if sse else None

        super().__init__(
            logical_bucket_name,
            bucket=physical_bucket_name,
            force_destroy=True,
            website=website_args,
            server_side_encryption_configuration=sse_config,
            # Ignoring force_destroy temporarily while we're
            # comparing/contrasting with CDK because otherwise it causes
            # noise in the diffs. It can be removed once we're fully in
            # Pulumi.
            opts=import_aware_opts(
                physical_bucket_name, parent=parent, ignore_changes=["forceDestroy"]
            ),
        )


def bucket_physical_name(logical_name: str) -> str:
    """Compute the physical name of a bucket, given its logical name.

    Mainly useful to help with resource importation logic on certain
    resources; may not be needed as a separate function once
    everything is managed by Pulumi.

    """
    return f"{DEPLOYMENT_NAME}-{logical_name}"


def sse_configuration() -> aws.s3.BucketServerSideEncryptionConfigurationArgs:
    """ Applies SSE to a bucket using AWS KMS. """
    return aws.s3.BucketServerSideEncryptionConfigurationArgs(
        rule=aws.s3.BucketServerSideEncryptionConfigurationRuleArgs(
            apply_server_side_encryption_by_default=aws.s3.BucketServerSideEncryptionConfigurationRuleApplyServerSideEncryptionByDefaultArgs(
                sse_algorithm="aws:kms",
            ),
        ),
    )
