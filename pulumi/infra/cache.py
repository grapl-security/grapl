from typing import Optional

import pulumi_aws as aws
from infra.network import Network

import pulumi


class Cache(pulumi.ComponentResource):
    """
    An ElastiCache cluster instance.
    """

    def __init__(
        self,
        name: str,
        network: Network,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:Cache", name, None, opts)

        redis_port = 6379

        self.subnet_group = aws.elasticache.SubnetGroup(
            f"{name}-cache-subnet-group",
            subnet_ids=[net.id for net in network.private_subnets],
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.security_group = aws.ec2.SecurityGroup(
            f"{name}-cache-security-group",
            vpc_id=network.vpc.id,
            # Defining ingress/egress rules inline here... this isn't
            # compatible right now with specifying standalone rules --
            # gotta pick one or the other.
            # (see
            # https://www.pulumi.com/docs/reference/pkg/aws/ec2/securitygrouprule)
            # NOTE: CDK code also had another ingress run allowing any
            # TCP connection from any source... leaving that off for
            # the time being.
            ingress=[
                aws.ec2.SecurityGroupIngressArgs(
                    description="Allow Redis connections from anywhere",
                    protocol="tcp",
                    from_port=redis_port,
                    to_port=redis_port,
                    # TODO: This just replicates what we're doing in
                    # CDK. Consider tightening this by using
                    # `security_groups` instead of `cidr_blocks`.
                    cidr_blocks=["0.0.0.0/0"],
                )
            ],
            opts=pulumi.ResourceOptions(parent=self),
        )

        # Note that this is a single-node Redis "cluster"
        # (a.k.a. "Cluster Mode Disabled")
        self.cluster = aws.elasticache.Cluster(
            f"{name}-cluster",
            engine="redis",
            port=redis_port,
            node_type="cache.t2.small",
            num_cache_nodes=1,
            subnet_group_name=self.subnet_group.name,
            security_group_ids=[self.security_group.id],
            opts=pulumi.ResourceOptions(parent=self),
        )

        # NOTE: This only works because we have single node
        # clusters. If we add more, we should abstract this in a
        # better way.

        # TODO: Why can't I export this?
        # self.address = pulumi.Output.concat(
        #     self.cluster.cache_nodes[0].address, ":", self.cluster.cache_nodes[0].port
        # )

        pulumi.export(f"{name}-cache-host", self.cluster.cache_nodes[0].address)
        pulumi.export(f"{name}-cache-port", self.cluster.cache_nodes[0].port)

        self.register_outputs({})
