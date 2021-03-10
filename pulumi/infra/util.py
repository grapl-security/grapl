from typing import Any

import pulumi_aws as aws

import pulumi

# Use this to modify behavior or configuration for provisioning in
# Local Grapl (as opposed to any other real deployment)
IS_LOCAL = pulumi.get_stack() == "local-grapl"


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
    prefix: str = pulumi.get_stack(),
    sse: bool = False,
    parent=None,
) -> aws.s3.Bucket:
    """Abstracts logic for creating an S3 bucket for our purposes.

    logical_bucket_name: What we call this bucket in Pulumi terms.

    prefix: an informative prefix to add to the logical bucket name to
    form a physical bucket name. Generally, this will be the Pulumi
    stack.

    sse: Whether or not to apply server-side encryption of
    bucket contents

    parent: for use in ComponentResources; the Pulumi resource
    that "owns" this resource.

    """
    physical_bucket_name = f"{prefix}-{logical_bucket_name}"
    base_args = {
        "bucket": physical_bucket_name,
        "opts": import_aware_opts(physical_bucket_name, parent=parent),
    }

    # TODO: Temporarily not doing encrypted buckets for Local
    # Grapl... I think we need to configure some stuff in that
    # environment a bit differently
    if sse and not IS_LOCAL:
        base_args["server_side_encryption_configuration"] = sse_config()

    return aws.s3.Bucket(logical_bucket_name, **base_args)


def sse_config():
    """ Applies SSE to a bucket using AWS KMS. """
    return aws.s3.BucketServerSideEncryptionConfigurationArgs(
        rule=aws.s3.BucketServerSideEncryptionConfigurationRuleArgs(
            apply_server_side_encryption_by_default=aws.s3.BucketServerSideEncryptionConfigurationRuleApplyServerSideEncryptionByDefaultArgs(
                sse_algorithm="aws:kms",
            ),
        ),
    )
