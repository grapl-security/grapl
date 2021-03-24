import os
from typing import Any, Optional

import pulumi_aws as aws

import pulumi

# This will be incorporated into various infrastructure object names.
DEPLOYMENT_NAME = pulumi.get_stack()

# Use this to modify behavior or configuration for provisioning in
# Local Grapl (as opposed to any other real deployment)
LOCAL_GRAPL = DEPLOYMENT_NAME == "local-grapl"

# For importing some objects, we have to construct a URL, ARN, etc
# that includes the AWS account ID.
AWS_ACCOUNT_ID = "000000000000" if LOCAL_GRAPL else aws.get_caller_identity().account_id

GLOBAL_LAMBDA_ZIP_TAG = os.getenv("GRAPL_LAMBDA_TAG", "latest")
"""Filename tag for all lambda function ZIP files.

All our lambda function ZIP files currently have a name like:

    <LAMBDA_NAME>-<TAG>.zip

Since all the lambdas share the same tag, we can specify this globally.

Use the environment variable `GRAPL_LAMBDA_TAG` to specify a tag, or
leave it unset to use the default value of `latest`.

"""


def mg_alphas() -> str:
    """Temporarily return a value to use as an `MG_ALPHAS` environment variable for services that need it until we pull Dgraph provisioning into Pulumi.

    This can be set explicitly on a stack-basis with the MG_ALPHAS
    config key. Otherwise, a value is constructed from the deployment
    name, as it was in our CDK code.

    """
    config = pulumi.Config()
    return config.get("MG_ALPHAS") or f"{DEPLOYMENT_NAME}.dgraph.grapl:9080"


# Yes I hate the 'Any' type just as much as you do, but there's
# apparently not a way to type kwargs right now.
def import_aware_opts(resource_id: str, **kwargs: Any) -> pulumi.ResourceOptions:
    """Pass the resource ID that corresponds to a particular resource
    when you're importing from existing infrastructure, as well as any
    other kwargs that `pulumi.ResourceOptions` would accept.

    If the Pulumi stack is currently configured to import (rather than
    create), the appropriate configuration will be injected into the
    `ResourceOptions` that is returned.

    Otherwise, a `ResourceOptions` constructed from the given kwargs
    will be returned.

    This should be used to generate `ResourceOptions` for *all* resources!

    To enable importing, rather than creating, set the following
    config for the stack:

        pulumi config set grapl:import_from_existing True

    """

    import_from_existing = pulumi.Config().require_bool("import_from_existing")

    given_opts = pulumi.ResourceOptions(**kwargs)
    import_opts = pulumi.ResourceOptions(
        import_=resource_id if import_from_existing else None
    )
    return pulumi.ResourceOptions.merge(given_opts, import_opts)


def grapl_bucket(
    logical_bucket_name: str,
    sse: bool = False,
    website_args: Optional[aws.s3.BucketWebsiteArgs] = None,
    parent: Optional[pulumi.Resource] = None,
) -> aws.s3.Bucket:
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

    # TODO: Temporarily not doing encrypted buckets for Local
    # Grapl... I think we may need to configure some stuff in
    # that environment a bit differently.
    sse_config = sse_configuration() if sse and not LOCAL_GRAPL else None

    return aws.s3.Bucket(
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
