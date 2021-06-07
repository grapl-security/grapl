from dataclasses import dataclass

import pulumi

import pulumi_aws as aws

TRAFFIC_FROM_ANYWHERE_CIDR = "0.0.0.0/0"


@dataclass
class Ec2Port:
    protocol: str
    port: int

    def allow_internally(
        self,
        sg: aws.ec2.SecurityGroup,
    ) -> None:
        aws.ec2.SecurityGroupRule(
            f"internal-ingress-{self}",
            type="ingress",
            security_group_id=sg.id,
            from_port=self.port,
            to_port=self.port,
            protocol=self.protocol,
            self=True,
            opts=pulumi.ResourceOptions(parent=sg),
        )

        aws.ec2.SecurityGroupRule(
            f"internal-egress-{self}",
            type="egress",
            security_group_id=sg.id,
            from_port=self.port,
            to_port=self.port,
            protocol=self.protocol,
            self=True,
            opts=pulumi.ResourceOptions(parent=sg),
        )

    def allow_outbound_any_ip(self, sg: aws.ec2.SecurityGroup) -> None:
        aws.ec2.SecurityGroupRule(
            f"outbound-any-ip-egress-{self}",
            type="egress",
            security_group_id=sg.id,
            from_port=self.port,
            to_port=self.port,
            protocol=self.protocol,
            cidr_blocks=[TRAFFIC_FROM_ANYWHERE_CIDR],
            opts=pulumi.ResourceOptions(parent=sg),
        )

    def __str__(self) -> str:
        return f"Ec2Port({self.protocol}:{self.port})"
