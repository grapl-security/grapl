import json

import pulumi_aws as aws

import pulumi


class Bucket(aws.s3.Bucket):
    def __init__(
        self,
        name: str,
        sse: bool = False,
        opts: pulumi.ResourceOptions | None = None,
    ) -> None:
        """Abstracts logic for creating an S3 bucket for our purposes.

        name: The Pulumi resource name. The physical bucket name will begin with this, and will receive a random suffix.

        sse: Whether or not to apply server-side encryption of
        bucket contents

        opts: `pulumi.ResourceOptions` for this resource.

        """
        sse_config = sse_configuration() if sse else None

        super().__init__(
            name,
            force_destroy=True,
            server_side_encryption_configuration=sse_config,
            opts=opts,
        )

    def grant_put_permission_to(self, role: aws.iam.Role) -> None:
        """Adds the ability to put objects into this bucket to the provided `Role`."""
        aws.iam.RolePolicy(
            f"{role._name}-writes-objects-to-{self._name}",
            role=role.name,
            policy=self.arn.apply(
                lambda bucket_arn: json.dumps(
                    {
                        "Version": "2012-10-17",
                        "Statement": [
                            {
                                "Effect": "Allow",
                                "Action": "s3:PutObject",
                                "Resource": f"{bucket_arn}/*",
                            }
                        ],
                    }
                )
            ),
            opts=pulumi.ResourceOptions(parent=role),
        )

    def grant_get_and_list_to(self, role: aws.iam.Role) -> None:
        """Grants GetObject on all the objects in the bucket, and ListBucket
        on the bucket itself.

        We may be able to use this for other permissions, but this was
        a specific structure ported over from our CDK code.

        NOTE: For SSM RunRemoteScript commands, use buckets with this grant.
        """
        aws.iam.RolePolicy(
            f"{role._name}-get-and-list-{self._name}",
            role=role.name,
            policy=self.arn.apply(
                lambda bucket_arn: json.dumps(
                    {
                        "Version": "2012-10-17",
                        "Statement": [
                            {
                                "Effect": "Allow",
                                "Action": "s3:GetObject",
                                "Resource": f"{bucket_arn}/*",
                            },
                            {
                                "Effect": "Allow",
                                "Action": "s3:ListBucket",
                                "Resource": bucket_arn,
                            },
                        ],
                    }
                )
            ),
            opts=pulumi.ResourceOptions(parent=role),
        )


def sse_configuration() -> aws.s3.BucketServerSideEncryptionConfigurationArgs:
    """Applies SSE to a bucket using AWS KMS."""
    return aws.s3.BucketServerSideEncryptionConfigurationArgs(
        rule=aws.s3.BucketServerSideEncryptionConfigurationRuleArgs(
            apply_server_side_encryption_by_default=aws.s3.BucketServerSideEncryptionConfigurationRuleApplyServerSideEncryptionByDefaultArgs(
                sse_algorithm="aws:kms",
            ),
        ),
    )
