import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Optional, Sequence, Tuple

import pulumi_aws as aws
from infra.bucket import Bucket
from infra.config import DEPLOYMENT_NAME, DGRAPH_LOG_RETENTION_DAYS
from infra.swarm import Ec2Port, Swarm
from pulumi_aws.ec2 import security_group
from pulumi_aws.ec2.vpc import Vpc

import pulumi
from pulumi.output import Output
from pulumi.resource import ResourceOptions

# These are COPYd in from Dockerfile.pulumi
DGRAPH_CONFIG_DIR = Path("../src/js/grapl-cdk/dgraph").resolve()


class DgraphCluster(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        vpc: Vpc,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:DgraphSwarmCluster", name=name, props=None, opts=opts)
        child_opts = pulumi.ResourceOptions(parent=self)

        self.log_group = aws.cloudwatch.LogGroup(
            f"{DEPLOYMENT_NAME}-grapl-dgraph",
            retention_in_days=DGRAPH_LOG_RETENTION_DAYS,
            opts=child_opts,
        )

        self.swarm = Swarm(
            name="swarm",
            vpc=vpc,
            log_group=self.log_group,
            internal_service_ports=[
                Ec2Port("tcp", x)
                for x in (
                    # Not 100% sure where these come from, I suspect they're
                    # DGraph alpha/zero port numbers.
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
        self.dgraph_config_bucket.grant_read_permissions_to(self.swarm.role)
        self.dgraph_config_bucket.upload_to_bucket(DGRAPH_CONFIG_DIR)

    def alpha_host_port(self) -> pulumi.Output[str]:
        return self.swarm.cluster_host_port()

    def allow_connections_from(self, other: Any) -> None:
        self.swarm.allow_connections_from(
            other, Ec2Port("tcp", 9080), opts=ResourceOptions(parent=self)
        )
