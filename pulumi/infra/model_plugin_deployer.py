from typing import Optional

from infra.bucket import Bucket
from infra.config import GLOBAL_LAMBDA_ZIP_TAG, LOCAL_GRAPL, configurable_envvars
from infra.dgraph_cluster import DgraphCluster
from infra.dynamodb import DynamoDB
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
        forwarder: Optional[MetricForwarder] = None,
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
            forwarder=forwarder,
            opts=pulumi.ResourceOptions(parent=self),
        )

        secret.grant_read_permissions_to(self.role)

        # TODO: Consider moving these permission-granting functions
        # into the dynamodb module (but still register them
        # here... just want to centralize the logic).

        # Read permissions for user auth DynamoDB table
        db.user_auth_table.grant_read_permissions_to(self.role)

        db.schema_table.grant_read_write_permissions_to(self.role)

        plugins_bucket.grant_read_write_permissions_to(self.role)

        plugins_bucket.grant_delete_permissions_to(self.role)

        dgraph_cluster.allow_connections_from(self.function.security_group)

        self.register_outputs({})
