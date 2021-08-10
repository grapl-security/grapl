from pulumi_policy import (
    EnforcementLevel,
    PolicyPack,
    ReportViolation,
    ResourceValidationArgs,
    ResourceValidationPolicy,
)


def s3_no_public_read_validator(args: ResourceValidationArgs, report_violation: ReportViolation):
    if args.resource_type == "aws:s3/bucket:Bucket" and "acl" in args.props:
        acl = args.props["acl"]
        if acl == "public-read" or acl == "public-read-write":
            report_violation(
                "You cannot set public-read or public-read-write on an S3 bucket. " +
                "Read more about ACLs here: https://docs.aws.amazon.com/AmazonS3/latest/dev/acl-overview.html")


def iam_no_s3_list_validator(args: ResourceValidationArgs, report_violation: ReportViolation):
    if args.resource_type == "aws:iam/rolepolicy:RolePolicy" and "policy" in args.props:
        if "s3:ListBucket" in args.props["policy"]:
            report_violation(
                "You should not require s3:ListBucket permissions"
            )


s3_no_public_read = ResourceValidationPolicy(
    name="s3-no-public-read",
    description="Prohibits setting the publicRead or publicReadWrite permission on AWS S3 buckets.",
    validate=s3_no_public_read_validator,
)

iam_no_s3_list = ResourceValidationPolicy(
    name="iam_no_s3_list",
    description="Prohibits setting the s3:ListBuckets permission in IAM policies",
    validate=iam_no_s3_list_validator,
)


PolicyPack(
    name="aws-python",
    enforcement_level=EnforcementLevel.MANDATORY,
    policies=[
        s3_no_public_read,
        iam_no_s3_list,
    ],
)
