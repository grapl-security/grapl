import json
from typing import Optional

import pulumi_aws as aws
from infra.config import DEPLOYMENT_NAME

import pulumi


class Bucket(aws.s3.Bucket):
    def __init__(
        self,
        logical_bucket_name: str,
        sse: bool = False,
        website_args: Optional[aws.s3.BucketWebsiteArgs] = None,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        """Abstracts logic for creating an S3 bucket for our purposes.

        logical_bucket_name: What we call this bucket in Pulumi terms.

        sse: Whether or not to apply server-side encryption of
        bucket contents

        website_args: configuration for setting the bucket up to serve web
        content.

        opts: `pulumi.ResourceOptions` for this resource.

        """
        physical_bucket_name = bucket_physical_name(logical_bucket_name)

        sse_config = sse_configuration() if sse else None

        super().__init__(
            logical_bucket_name,
            bucket=physical_bucket_name,
            force_destroy=True,
            website=website_args,
            server_side_encryption_configuration=sse_config,
            opts=opts,
        )

    def grant_read_permissions_to(self, role: aws.iam.Role) -> None:
        """ Adds the ability to read from this bucket to the provided `Role`. """
        aws.iam.RolePolicy(
            f"{role._name}-reads-{self._name}",
            role=role.name,
            policy=self.arn.apply(
                lambda bucket_arn: json.dumps(
                    {
                        "Version": "2012-10-17",
                        "Statement": [
                            {
                                "Effect": "Allow",
                                "Action": [
                                    # TODO: Prefer to enumerate specific
                                    # actions rather than wildcards
                                    "s3:GetObject*",
                                    "s3:GetBucket*",
                                    "s3:List*",
                                ],
                                "Resource": [bucket_arn, f"{bucket_arn}/*"],
                            }
                        ],
                    }
                )
            ),
            opts=pulumi.ResourceOptions(parent=self),
        )

    def grant_read_write_permissions_to(self, role: aws.iam.Role) -> None:
        """ Gives the provided `Role` the ability to read from and write to this bucket. """
        aws.iam.RolePolicy(
            f"{role._name}-reads-and-writes-{self._name}",
            role=role.name,
            policy=self.arn.apply(
                lambda bucket_arn: json.dumps(
                    {
                        "Version": "2012-10-17",
                        "Statement": [
                            {
                                "Effect": "Allow",
                                # TODO: Prefer to split
                                # these up by bucket /
                                # object, as well as
                                # enumerate the *specific*
                                # actions that are needed.
                                "Action": [
                                    "s3:GetObject*",
                                    "s3:GetBucket*",
                                    "s3:List*",
                                    "s3:DeleteObject*",
                                    "s3:PutObject*",
                                    "s3:Abort*",
                                ],
                                "Resource": [bucket_arn, f"{bucket_arn}/*"],
                            },
                        ],
                    }
                )
            ),
            opts=pulumi.ResourceOptions(parent=self),
        )

    def grant_delete_permissions_to(self, role: aws.iam.Role) -> None:
        """ Adds the ability to delete objects from this bucket to the provided `Role`. """
        aws.iam.RolePolicy(
            f"{role._name}-deletes-{self._name}",
            role=role.name,
            policy=self.arn.apply(
                lambda bucket_arn: json.dumps(
                    {
                        "Version": "2012-10-17",
                        "Statement": [
                            {
                                "Effect": "Allow",
                                # TODO: Prefer to enumerate specific
                                # actions rather than wildcards
                                "Action": "s3:DeleteObject*",
                                "Resource": f"{bucket_arn}/*",
                            }
                        ],
                    }
                )
            ),
            opts=pulumi.ResourceOptions(parent=self),
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
