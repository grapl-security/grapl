from typing import Optional

import pulumi_aws as aws
from infra.config import GLOBAL_LAMBDA_ZIP_TAG
from infra.dgraph_cluster import DgraphCluster
from infra.lambda_ import Lambda, LambdaExecutionRole, PythonLambdaArgs, code_path_for
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
                description=GLOBAL_LAMBDA_ZIP_TAG,
                handler="lambdex_handler.handler",
                code_path=code_path_for(name),
                env={
                    "GRAPL_LOG_LEVEL": "INFO",
                    "MG_ALPHAS": dgraph_cluster.alpha_host_port(),
                    "GRAPL_DGRAPH_TTL_S": str(60 * 60 * 24 * 31),  # 1 month
                    "GRAPL_TTL_DELETE_BATCH_SIZE": "1000",
                },
                memory_size=128,
                timeout=600,
            ),
            network=network,
            opts=pulumi.ResourceOptions(parent=self),
        )

        # TODO: Need to allow connections to DGraph from this lambda

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

        self.register_outputs({})
