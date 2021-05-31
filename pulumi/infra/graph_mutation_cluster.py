from pathlib import Path
from typing import Any, Optional, Final

from infra import dynamodb

import pulumi
import pulumi_aws as aws
from pulumi.output import Output

from infra.bucket import Bucket
from infra.config import DEPLOYMENT_NAME
from infra.dynamodb import DynamoDB
from infra.grpc_swarm import Ec2Port, GrpcSwarm

# These are COPYd in from Dockerfile.pulumi
_CONFIG_DIR: Final[Path] = Path("../src/js/grapl-cdk/graph_mutation_service").resolve()


class GraphMutationCluster(pulumi.ComponentResource):
    def __init__(
            self,
            vpc: aws.ec2.Vpc,
            db: DynamoDB,
            opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:GrpcSwarmCluster", name='graph-mutation-service', props=None, opts=opts)
        child_opts = pulumi.ResourceOptions(parent=self)

        self.swarm = GrpcSwarm(
            name=f"{DEPLOYMENT_NAME}-graph-mutation-service-swarm",
            service_name='graph-mutation-service',
            vpc=vpc,
            internal_service_ports=[
                Ec2Port("tcp", 5500),
                Ec2Port("tcp", 5501),
            ],
            opts=child_opts,
        )

        dynamodb.grant_read_on_tables(
            self.swarm.role, [db.schema_table, db.schema_properties_table]
        )

        self.service_config_bucket = Bucket(
            logical_bucket_name="graph-mutation-service-config-bucket",
            opts=child_opts,
        )
        self.service_config_bucket.grant_read_permissions_to(self.swarm.role)
        self.service_config_bucket.upload_to_bucket(_CONFIG_DIR)

        self.register_outputs({})

    @property
    def alpha_host_port(self) -> pulumi.Output[str]:
        # endpoint might be a better name
        return self.swarm.cluster_host_port

    def allow_connections_from(self, other: aws.ec2.SecurityGroup) -> None:
        """
        Need to pass in a lambda? Access its `.function.security_group`
        """
        self.swarm.allow_connections_from(other, Ec2Port("tcp", 5500))


class LocalStandInGraphMutationCluster(GraphMutationCluster):
    """
    We can't use the real GraphMutationCluster object yet because
    we are in this about-to-kill-off-local-grapl limbo world.

    However, I still want to feed an object matching its API into
    other lambdas, to replace `mg_alphas()`.
    """

    def __init__(self, *args: Any, **kwargs: Any) -> None:
        pass

    @property
    def alpha_host_port(self) -> pulumi.Output[str]:
        config = pulumi.Config()
        endpoint = config.get("MG_ALPHAS") or f"{DEPLOYMENT_NAME}.graph-mutation-service.grapl:5500"
        return Output.from_input(endpoint)

    def allow_connections_from(self, other: aws.ec2.SecurityGroup) -> None:
        pass
