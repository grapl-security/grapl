
import pulumi_aws as aws
import pulumi

class SecurityGroup(pulumi.ComponentResource):
    def __init__(self, name: str, opts: pulumi.ResourceOptions=None) -> None:
        super().__init__("devbox:SecurityGroup", name=name, props=None, opts=opts)
        security_group_name = "devbox-security-group"
        self.security_group = aws.ec2.SecurityGroup(
            security_group_name,
            vpc_id=None,
            # Tags are necessary for the moment so we can look up the resource from a different pulumi stack.
            # Once this is refactored we can remove the tags
            tags={"Name": security_group_name},
            opts=pulumi.ResourceOptions(parent=self),
        )

        # The way SSM works is wild. An agent on the box reaches out to a known
        # IP on 443, and then establishes a bidirectional pipe.
        # As such, all you need to enable is 443 outbound.
        aws.ec2.SecurityGroupRule(
            f"ssm_egress",
            type="egress",
            security_group_id=self.security_group.id,
            from_port=0,
            to_port=65535,
            protocol="tcp",
            cidr_blocks=["0.0.0.0/0"],
            opts=pulumi.ResourceOptions(parent=self),
        )