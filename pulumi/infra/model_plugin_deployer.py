from typing import Optional

from infra import dynamodb
from infra.bucket import Bucket
from infra.config import GLOBAL_LAMBDA_ZIP_TAG, LOCAL_GRAPL, configurable_envvars
from infra.dgraph_cluster import DgraphCluster
from infra.dynamodb import DynamoDB
from infra.ec2 import Ec2Port
from infra.lambda_ import Lambda, LambdaExecutionRole, PythonLambdaArgs, code_path_for
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
        ux_bucket: Bucket,
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
                description=GLOBAL_LAMBDA_ZIP_TAG,
                handler="lambdex_handler.handler",
                code_path=code_path_for(name),
                env={
                    **configurable_envvars(name, ["GRAPL_LOG_LEVEL"]),
                    "MG_ALPHAS": dgraph_cluster.alpha_host_port,
                    "JWT_SECRET_ID": secret.secret.arn
                    if not LOCAL_GRAPL
                    else "JWT_SECRET_ID",
                    "USER_AUTH_TABLE": db.user_auth_table.id,
                    "DEPLOYMENT_NAME": pulumi.get_stack(),
                    "UX_BUCKET_URL": pulumi.Output.concat(
                        "https://", ux_bucket.bucket_regional_domain_name
                    ),
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

        dynamodb.grant_read_on_tables(self.role, [db.user_auth_table])
        dynamodb.grant_read_write_on_tables(self.role, [db.schema_table])

        plugins_bucket.grant_read_write_permissions_to(self.role)

        dgraph_cluster.allow_connections_from(self.function.security_group)

        self.register_outputs({})
