from typing import List, Optional

import pulumi_aws as aws
from infra.config import DEPLOYMENT_NAME

import pulumi


class Cache(pulumi.ComponentResource):
    """
    An ElastiCache cluster instance.
    """

    def __init__(
        self,
        name: str,
        subnet_ids: pulumi.Input[List[str]],
        vpc_id: pulumi.Input[str],
        nomad_agent_security_group_id: pulumi.Input[str],
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:Cache", name, None, opts)

        redis_port = 6379

        self.subnet_group = aws.elasticache.SubnetGroup(
            f"{name}-cache-subnet-group",
            subnet_ids=subnet_ids,
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.security_group = aws.ec2.SecurityGroup(
            f"{name}-cache-security-group",
            vpc_id=vpc_id,
            # Tags are necessary for the moment so we can look up the resource from a different pulumi stack.
            # Once this is refactored we can remove the tags
            tags={"Name": f"{name}-{DEPLOYMENT_NAME}"},
            opts=pulumi.ResourceOptions(parent=self),
        )

        # Allow communication between nomad-agents and redis
        # These are in different VPCs with the peering done in the networking module
        # We assume that the stack name for nomad-agents is the same as grapl's stack name
        aws.ec2.SecurityGroupRule(
            "nomad-agents-egress-to-redis",
            type="egress",
            security_group_id=nomad_agent_security_group_id,
            from_port=redis_port,
            to_port=redis_port,
            protocol="tcp",
            source_security_group_id=self.security_group.id,
            opts=pulumi.ResourceOptions(parent=self.security_group),
        )

        aws.ec2.SecurityGroupRule(
            "redis-ingress-from-nomad-agents",
            type="ingress",
            security_group_id=self.security_group.id,
            from_port=redis_port,
            to_port=redis_port,
            protocol="tcp",
            source_security_group_id=nomad_agent_security_group_id,
            opts=pulumi.ResourceOptions(parent=self.security_group),
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
