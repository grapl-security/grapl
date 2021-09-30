from typing import Optional

from infra import dynamodb
from infra.bucket import Bucket
from infra.config import LOCAL_GRAPL, configurable_envvars
from infra.dgraph_cluster import DgraphCluster
from infra.dynamodb import DynamoDB
from infra.ec2 import Ec2Port
from infra.lambda_ import Lambda, LambdaExecutionRole, LambdaResolver, PythonLambdaArgs
from infra.metric_forwarder import MetricForwarder
from infra.network import Network
from infra.secret import JWTSecret

import pulumi


class ModelPluginDeployer(pulumi.ComponentResource):
    def __init__(
        self,
        network: Network,
        db: DynamoDB,
        secret: JWTSecret,
        plugins_bucket: Bucket,
        dgraph_cluster: DgraphCluster,
        forwarder: MetricForwarder,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:

        name = "model-plugin-deployer"
        super().__init__("grapl:ModelPluginDeployer", name, None, opts)

        self.role = LambdaExecutionRole(name, opts=pulumi.ResourceOptions(parent=self))

        self.function = Lambda(
            name,
            args=PythonLambdaArgs(
                execution_role=self.role,
                handler="lambdex_handler.handler",
                code=LambdaResolver.resolve(name),
                env={
                    **configurable_envvars(name, ["GRAPL_LOG_LEVEL"]),
                    "GRAPL_MODEL_PLUGINS_BUCKET": plugins_bucket.bucket,
                    "MG_ALPHAS": dgraph_cluster.alpha_host_port,
                    "JWT_SECRET_ID": secret.secret.arn
                    if not LOCAL_GRAPL
                    else "JWT_SECRET_ID",
                    "GRAPL_SCHEMA_PROPERTIES_TABLE": db.schema_properties_table.id,
                    "GRAPL_SCHEMA_TABLE": db.schema_table.id,
                },
                timeout=25,
                memory_size=256,
            ),
            network=network,
            opts=pulumi.ResourceOptions(parent=self),
        )

        Ec2Port("tcp", 443).allow_outbound_any_ip(self.function.security_group)

        forwarder.subscribe_to_log_group(name, self.function.log_group)

        secret.grant_read_permissions_to(self.role)

        dynamodb.grant_read_write_on_tables(
            self.role, [db.schema_table, db.schema_properties_table]
        )

        plugins_bucket.grant_read_write_permissions_to(self.role)

        dgraph_cluster.allow_connections_from(self.function.security_group)

        self.register_outputs({})
