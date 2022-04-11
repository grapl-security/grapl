import sys

sys.path.insert(0, ".")

import pulumi_aws as aws
from infra.ami import get_ami

import pulumi


def main() -> None:
    config = pulumi.Config()
    instance_type = config.require("instance-type")

    security_group_name = "devbox-security-group"
    security_group = aws.ec2.SecurityGroup(
        security_group_name,
        vpc_id=None,
        # Tags are necessary for the moment so we can look up the resource from a different pulumi stack.
        # Once this is refactored we can remove the tags
        tags={"Name": security_group_name},
    )

    ami = get_ami()


if __name__ == "__main__":
    main()
