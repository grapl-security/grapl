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
            tags={"Name": f"{name}"},
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

        self.register_outputs({})

    @property
    def endpoint(self) -> pulumi.Output[str]:
        """
        Return an endpoint URL for accessing this cache from other services.

        Uses the "redis://" protocol.
        """
        # NOTE: This only works because we have single node
        # clusters. If we add more, we should expose them all in a
        # better way.
        return pulumi.Output.all(
            host=self.host,  # type: ignore[arg-type]
            port=self.port,  # type: ignore[arg-type]
        ).apply(lambda args: f"redis://{args['host']}:{args['port']}")

    @property
    def host(self) -> pulumi.Output[str]:
        """
        Returns the host of the first (and only) node in the cluster.
        """
        return self.cluster.cache_nodes[0].address  # type: ignore[no-any-return]

    @property
    def port(self) -> pulumi.Output[int]:
        """
        Returns the port of the first (and only) node in the cluster.
        """
        return self.cluster.cache_nodes[0].port  # type: ignore[no-any-return]

    def allow_egress_to_cache_for(
        self, name: str, origin: aws.ec2.SecurityGroup
    ) -> None:
        """
        Create an egress rule for the `origin` security group, allowing communication to the cache's port.

        The security group rule will be a child of the `origin` security group Pulumi resource.

        `name` is a descriptive string that will be incorporated into the Pulumi resource name of the security group rule.
        """
        aws.ec2.SecurityGroupRule(
            f"{name}-egress-to-cache",
            type="egress",
            description=self.cluster.id.apply(
                lambda id: f"Allow outbound traffic to Redis cluster {id}"
            ),
            from_port=self.port,
            to_port=self.port,
            protocol=aws.ec2.ProtocolType.TCP,
            security_group_id=origin.id,
            source_security_group_id=self.security_group.id,
            opts=pulumi.ResourceOptions(parent=origin),
        )
