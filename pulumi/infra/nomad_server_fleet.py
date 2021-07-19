import json
from typing import Optional, Sequence

import pulumi_aws as aws
from infra import policies
from infra.ami_artifacts import get_ami_id
from infra.config import DEPLOYMENT_NAME, DGRAPH_LOG_RETENTION_DAYS
from infra.ec2 import Ec2Port
from infra.ec2_cluster import Ec2Cluster
from infra.network import Network
from infra.policies import EC2_DESCRIBE_INSTANCES_POLICY

import pulumi


class NomadServerFleet(pulumi.ComponentResource):
    """
    NOTE: We also currently bake in the Consul server into the Nomad server;
    they are colocated.
    The image is defined in `image.pkr.hcl` and is based on:
    https://github.com/hashicorp/terraform-aws-nomad/tree/master/examples/nomad-consul-ami

    This decision can be revisited, in which case you'd want to look at
    https://github.com/hashicorp/terraform-aws-nomad/tree/master/examples/nomad-consul-separate-cluster
    """

    def __init__(
        self,
        name: str,
        network: Network,
        internal_service_ports: Sequence[Ec2Port],
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:NomadServerFleet", name=name, props=None, opts=opts)
        child_opts = pulumi.ResourceOptions(parent=self)
        vpc = network.vpc

        self.log_group = aws.cloudwatch.LogGroup(
            f"{name}-logs",
            retention_in_days=DGRAPH_LOG_RETENTION_DAYS,  # TODO: ??
            opts=child_opts,
        )

        self.security_group = aws.ec2.SecurityGroup(
            f"{name}-sec-group",
            description=f"Nomad server security group",
            vpc_id=vpc.id,
            opts=child_opts,
        )

        self.role = aws.iam.Role(
            f"{name}-role",
            description="IAM role for Nomad server instances",
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

        self._open_initial_ports(internal_service_ports)

        # InstanceProfile for nomad-server instances
        instance_profile = aws.iam.InstanceProfile(
            f"{name}-instance-profile",
            opts=child_opts,
            role=self.role.name,
            name=f"{DEPLOYMENT_NAME}-nomad-server-instance-profile",
        )

        # CloudWatchAgentServerPolicy allows the Nomad server instances to
        # run the CloudWatch Agent.
        policies.attach_policy(
            role=self.role, policy=policies.CLOUDWATCH_AGENT_SERVER_POLICY
        )
        policies.attach_policy(role=self.role, policy=policies.SSM_POLICY)

        policies.attach_policy(role=self.role, policy=EC2_DESCRIBE_INSTANCES_POLICY)

        nomad_server_ami = get_ami_id("grapl-nomad-consul-server")

        nomad_servers = Ec2Cluster(
            name="nomad-servers",
            vpc=network,
            quorum_size=3,
            quorums=1,
            ami=nomad_server_ami,
            instance_type="t2.micro",
            iam_instance_profile=instance_profile,
            vpc_security_group_ids=[self.security_group.id],
            instance_tags={
                "nomad-server-sec-group-for-deployment": DEPLOYMENT_NAME,
                "ConsulAgent": "Server",
            },
            opts=child_opts,
        )

    def _open_initial_ports(self, internal_service_ports: Sequence[Ec2Port]) -> None:
        # allow hosts in the nomad-server security group to communicate
        # internally on the following ports:
        # https://www.nomadproject.io/docs/install/production/requirements#ports-used
        # https://www.consul.io/docs/install/ports
        for port in (
            # Nomad ports
            Ec2Port("tcp", 4646),
            Ec2Port("tcp", 4647),
            Ec2Port("tcp", 4648),
            Ec2Port("udp", 4648),
            # Consul ports
            Ec2Port("tcp", 8300),
            Ec2Port("tcp", 8301),
            Ec2Port("udp", 8301),
            Ec2Port("tcp", 8302),
            Ec2Port("udp", 8302),
            Ec2Port("tcp", 8500),
            Ec2Port("udp", 8600),
            Ec2Port("tcp", 8600),
            *internal_service_ports,
        ):
            port.allow_internally(self.security_group)

        # allow hosts in the nomad-server security group to make outbound
        # connections to the Internet for these services:
        #   TCP 443 -- AWS SSM Agent (for handshake)
        #   TCP 80 -- yum package manager and wget
        for port in (
            Ec2Port("tcp", 443),
            Ec2Port("tcp", 80),
        ):
            port.allow_outbound_any_ip(self.security_group)

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
            # Awful naming scheme
            # https://grapl-internal.slack.com/archives/C017DKYF55H/p1621459760032000
            # For egress, `source` is where it's going
            source_security_group_id=self.security_group.id,
            security_group_id=other.id,
            from_port=port_range.port,
            to_port=port_range.port,
            protocol=port_range.protocol,
            opts=pulumi.ResourceOptions(parent=self.security_group),
        )
