import json
from dataclasses import dataclass
from pathlib import Path
from typing import Optional, Sequence, Tuple

import pulumi_aws as aws
from infra import policies
from infra.bucket import Bucket
from infra.config import DEPLOYMENT_NAME, DGRAPH_LOG_RETENTION_DAYS

import pulumi
from pulumi.output import Output
from pulumi.resource import ResourceOptions

# These are COPYd in from Dockerfile.pulumi
SWARM_INIT_DIR = Path("../src/js/grapl-cdk/swarm").resolve()

TRAFFIC_FROM_ANYWHERE_CIDR = "0.0.0.0/0"


@dataclass
class Ec2Port:
    protocol: str
    port: int

    def allow_internally(
        self,
    ) -> Tuple[aws.ec2.SecurityGroupIngressArgs, aws.ec2.SecurityGroupEgressArgs]:
        ingress = aws.ec2.SecurityGroupIngressArgs(
            from_port=self.port,
            to_port=self.port,
            protocol=self.protocol,
            self=True,
        )
        egress = aws.ec2.SecurityGroupEgressArgs(
            from_port=self.port,
            to_port=self.port,
            protocol=self.protocol,
            self=True,
        )
        return (ingress, egress)

    def allow_outbound_any_ip(self) -> aws.ec2.SecurityGroupEgressArgs:
        return aws.ec2.SecurityGroupEgressArgs(
            from_port=self.port,
            to_port=self.port,
            protocol=self.protocol,
            cidr_blocks=[TRAFFIC_FROM_ANYWHERE_CIDR],
        )

    def __str__(self) -> str:
        return f"Ec2Port({self.protocol}:{self.port})"


class Swarm(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        vpc: aws.ec2.Vpc,
        internal_service_ports: Sequence[Ec2Port],
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:SwarmResource", name=name, props=None, opts=opts)

        child_opts = pulumi.ResourceOptions(parent=self)

        self.log_group = aws.cloudwatch.LogGroup(
            f"{name}-logs",
            retention_in_days=DGRAPH_LOG_RETENTION_DAYS,
            opts=child_opts,
        )

        # allow hosts in the swarm security group to communicate
        # internally on the following ports:
        #   TCP 2376 -- secure docker client
        #   TCP 2377 -- inter-node communication (only needed on manager nodes)
        #   TCP + UDP 7946 -- container network discovery
        #   UDP 4789 -- overlay network traffic
        internal_rules = [
            port.allow_internally()
            for port in (
                Ec2Port("tcp", 2376),
                Ec2Port("tcp", 2377),
                Ec2Port("tcp", 7946),
                Ec2Port("udp", 7946),
                Ec2Port("udp", 4789),
                *internal_service_ports,
            )
        ]

        ingress_rules = [ingress for ingress, _ in internal_rules]

        # allow hosts in the swarm security group to make outbound
        # connections to the Internet for these services:
        #   TCP 443 -- AWS SSM Agent (for handshake)
        #   TCP 80 -- yum package manager and wget (install Docker)
        egress_rules = [
            port.allow_outbound_any_ip()
            for port in (
                Ec2Port("tcp", 443),
                Ec2Port("tcp", 80),
            )
        ] + [egress for _, egress in internal_rules]

        self.security_group = aws.ec2.SecurityGroup(
            f"{name}-sec-group",
            description=f"Docker Swarm security group",
            vpc_id=vpc.id,
            tags={"swarm-sec-group-for-deployment": f"{DEPLOYMENT_NAME}"},
            opts=child_opts,
        )

        self.role = aws.iam.Role(
            f"{name}-role",
            description="IAM role for Swarm instances",
            assume_role_policy=json.dumps(
                {
                    "Version": "2012-10-17",
                    "Statement": [
                        {
                            "Action": "sts:AssumeRole",
                            "Effect": "Allow",
                            "Principal": {
                                "Service": "ec2.amazonaws.com",
                            },
                        }
                    ],
                }
            ),
            opts=child_opts,
        )

        # InstanceProfile for swarm instances
        aws.iam.InstanceProfile(
            f"{name}-instance-profile",
            opts=child_opts,
            role=self.role.name,
            name=f"{DEPLOYMENT_NAME}-swarm-instance-profile",
        )

        # CloudWatchAgentServerPolicy allows the Swarm instances to
        # run the CloudWatch Agent.
        policies.attach_policy(
            role=self.role, policy=policies.CLOUDWATCH_AGENT_SERVER_POLICY
        )
        policies.attach_policy(role=self.role, policy=policies.SSM_POLICY)
        policies._attach_policy_to_ship_logs_to_cloudwatch(
            role=self.role, log_group=self.log_group, opts=child_opts
        )

        self.swarm_hosted_zone = aws.route53.Zone(
            f"{name}-hosted-zone",
            name=f"{DEPLOYMENT_NAME}.dgraph.grapl",
            vpcs=[
                aws.route53.ZoneVpcArgs(
                    vpc_id=vpc.id,
                )
            ],
            opts=child_opts,
        )

        self.swarm_config_bucket = Bucket(
            logical_bucket_name="swarm-config-bucket",
            opts=child_opts,
        )
        self.swarm_config_bucket.grant_read_permissions_to(self.role)
        self.swarm_config_bucket.upload_to_bucket(SWARM_INIT_DIR)

        self.register_outputs({})

    @property
    def cluster_host_port(self) -> pulumi.Output[str]:
        return Output.concat(self.swarm_hosted_zone.name, ":9080")

    def allow_connections_from(
        self,
        other: aws.ec2.SecurityGroup,
        port_range: Ec2Port,
    ) -> None:
        descriptor = "-".join(
            [
                "from",
                other._name,
                "for",
                str(port_range),
            ]
        )

        # We'll accept connections from Other into SecurityGroup
        aws.ec2.SecurityGroupRule(
            f"ingress-{descriptor}",
            type="ingress",
            source_security_group_id=other.id,
            security_group_id=self.security_group.id,
            from_port=port_range.port,
            to_port=port_range.port,
            protocol=port_range.protocol,
            opts=pulumi.ResourceOptions(parent=self.security_group),
        )

        # Allow connections out of Other to Self
        aws.ec2.SecurityGroupRule(
            f"egress-{descriptor}",
            type="egress",
            # Perhaps the source security group is wrong
            # https://grapl-internal.slack.com/archives/C017DKYF55H/p1621030148014300
            # Comparing with CDK output, seems semi right
            source_security_group_id=self.security_group.id,
            security_group_id=other.id,
            from_port=port_range.port,
            to_port=port_range.port,
            protocol=port_range.protocol,
            opts=pulumi.ResourceOptions(parent=self.security_group),
        )
