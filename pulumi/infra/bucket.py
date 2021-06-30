import json
from pathlib import Path
from typing import List, Optional

import pulumi_aws as aws
from infra.config import DEPLOYMENT_NAME
from pulumi.resource import ResourceOptions

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
        self.logical_bucket_name = logical_bucket_name

        sse_config = sse_configuration() if sse else None

        super().__init__(
            logical_bucket_name,
            bucket=bucket_physical_name(logical_bucket_name),
            force_destroy=True,
            website=website_args,
            server_side_encryption_configuration=sse_config,
            opts=opts,
        )

    def grant_read_permission_to(self, role: aws.iam.Role) -> None:
        """ Adds the ability to read objects from this bucket to the provided `Role`. """
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
                                "Action": "s3:GetObject",
                                "Resource": f"{bucket_arn}/*",
                            }
                        ],
                    }
                )
            ),
            opts=pulumi.ResourceOptions(parent=role),
        )

    def grant_put_permission_to(self, role: aws.iam.Role) -> None:
        """ Adds the ability to put objects into this bucket to the provided `Role`. """
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
                                "Action": "s3:ListBucket",
                                "Resource": bucket_arn,
                            },
                            {
                                "Effect": "Allow",
                                "Action": [
                                    "s3:GetObject",
                                    "s3:DeleteObject",
                                    "s3:PutObject",
                                ],
                                "Resource": f"{bucket_arn}/*",
                            },
                        ],
                    }
                )
            ),
            opts=pulumi.ResourceOptions(parent=role),
        )

    def _upload_file_to_bucket(
        self, file_path: Path, root_path: Path
    ) -> aws.s3.BucketObject:
        """ Compare with CDK's s3deploy.BucketDeployment """
        assert (
            file_path.is_file()
        ), f"Use `upload_dir_to_bucket` for directory {file_path}"
        name = str(file_path.relative_to(root_path))
        return aws.s3.BucketObject(
            name,
            bucket=self.id,
            source=pulumi.FileAsset(file_path),
            opts=ResourceOptions(parent=self)
            # Do we need to specify mimetype?
        )

    def upload_to_bucket(
        self,
        file_path: Path,
        root_path: Optional[Path] = None,
    ) -> List[aws.s3.BucketObject]:
        """
        Compare with CDK's s3deploy.BucketDeployment
        root_path is so that:

        given file_path="someplace/some_dir", root_path = "someplace"
        the uploaded files can be named
        "some_dir/a.txt"
        "some_dir/b.txt"
        "some_dir/subdir/c.txt"
        basically, the `root_path` becomes the `/` on the s3 side
        """
        if file_path.is_file():
            root_path = root_path or file_path.parent
            return [self._upload_file_to_bucket(file_path, root_path=root_path)]
        elif file_path.is_dir():
            root_path = root_path or file_path
            # Flattens it
            return sum(
                (
                    self.upload_to_bucket(child, root_path=root_path)
                    for child in file_path.iterdir()
                ),
                [],
            )
        else:
            raise FileNotFoundError(
                f"Neither a file nor a dir - does it exist?: {file_path}"
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
