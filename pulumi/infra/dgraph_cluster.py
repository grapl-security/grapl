from typing import Any, Optional

import pulumi_aws as aws
from infra.bucket import Bucket
from infra.config import repository_path
from infra.ec2 import Ec2Port
from infra.swarm import Swarm
from pulumi.output import Output

import pulumi

# These are COPYd in from Dockerfile.pulumi
DGRAPH_CONFIG_DIR = repository_path("src/aws-provision/dgraph")


class DgraphCluster(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        vpc: aws.ec2.Vpc,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:DgraphSwarmCluster", name=name, props=None, opts=opts)
        child_opts = pulumi.ResourceOptions(parent=self)

        self.swarm = Swarm(
            name=f"{name}-swarm",
            vpc=vpc,
            internal_service_ports=[
                Ec2Port("tcp", x)
                for x in (
                    # DGraph alpha/zero port numbers
                    # https://dgraph.io/docs/deploy/dgraph-zero/
                    5080,
                    6080,
                    7081,
                    7082,
                    7083,
                    8081,
                    8082,
                    8083,
                    9081,
                    9082,
                    9083,
                )
            ],
            opts=child_opts,
        )

        self.dgraph_config_bucket = Bucket(
            logical_bucket_name="dgraph-config-bucket",
            opts=child_opts,
        )
        pulumi.export("dgraph-config-bucket", self.dgraph_config_bucket.bucket)

        self.dgraph_config_bucket.grant_get_and_list_to(self.swarm.role)
        self.dgraph_config_bucket.upload_to_bucket(DGRAPH_CONFIG_DIR)

        self.register_outputs({})

    @property
    def alpha_host_port(self) -> pulumi.Output[str]:
        # endpoint might be a better name
        return self.swarm.cluster_host_port

    def allow_connections_from(self, other: aws.ec2.SecurityGroup) -> None:
        """
        Need to pass in a lambda? Access its `.function.security_group`
        """
        self.swarm.allow_connections_from(other, Ec2Port("tcp", 9080))


class LocalStandInDgraphCluster(DgraphCluster):
    """
    We can't use the real DgraphCluster object yet because
    we are in this about-to-kill-off-local-grapl limbo world.

    However, I still want to feed an object matching its API into
    other lambdas, to replace `mg_alphas()`.
    """

    def __init__(self, *args: Any, **kwargs: Any) -> None:
        pass

    @property
    def alpha_host_port(self) -> pulumi.Output[str]:
        config = pulumi.Config()
        endpoint = config.require("MG_ALPHAS")
        return Output.from_input(endpoint)

    def allow_connections_from(self, other: aws.ec2.SecurityGroup) -> None:
        pass
