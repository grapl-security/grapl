from typing import Optional

import pulumi_aws as aws
from infra.config import configurable_envvars
from infra.dgraph_cluster import DgraphCluster
from infra.ec2 import Ec2Port
from infra.lambda_ import Lambda, LambdaExecutionRole, LambdaResolver, PythonLambdaArgs
from infra.network import Network

import pulumi


class DGraphTTL(pulumi.ComponentResource):
    def __init__(
        self,
        network: Network,
        dgraph_cluster: DgraphCluster,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:

        name = "dgraph-ttl"
        super().__init__("grapl:DGraphTTL", name, None, opts)

        self.role = LambdaExecutionRole(
            name,
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.function = Lambda(
            f"{name}-Handler",
            args=PythonLambdaArgs(
                execution_role=self.role,
                handler="lambdex_handler.handler",
                code=LambdaResolver.resolve(name),
                env={
                    **configurable_envvars(name, ["GRAPL_LOG_LEVEL"]),
                    "MG_ALPHAS": dgraph_cluster.alpha_host_port,
                    "GRAPL_DGRAPH_TTL_S": str(60 * 60 * 24 * 31),  # 1 month
                    "GRAPL_TTL_DELETE_BATCH_SIZE": "1000",
                },
                memory_size=128,
                timeout=600,
            ),
            network=network,
            opts=pulumi.ResourceOptions(parent=self),
        )

        Ec2Port("tcp", 443).allow_outbound_any_ip(self.function.security_group)

        self.scheduling_rule = aws.cloudwatch.EventRule(
            f"{name}-hourly-trigger",
            schedule_expression="rate(1 hour)",
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.target = aws.cloudwatch.EventTarget(
            f"{name}-invocation-target",
            arn=self.function.function.arn,
            rule=self.scheduling_rule.name,
            opts=pulumi.ResourceOptions(parent=self),
        )

        dgraph_cluster.allow_connections_from(self.function.security_group)

        self.register_outputs({})
