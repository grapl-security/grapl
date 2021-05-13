from typing import Optional

import pulumi_aws as aws
from infra.bucket import Bucket
from infra.config import GLOBAL_LAMBDA_ZIP_TAG, LOCAL_GRAPL
from infra.dgraph_cluster import DgraphCluster
from infra.lambda_ import Lambda, LambdaArgs, LambdaExecutionRole, code_path_for
from infra.metric_forwarder import MetricForwarder
from infra.network import Network
from infra.secret import JWTSecret

import pulumi


class GraphQL(pulumi.ComponentResource):
    def __init__(
        self,
        secret: JWTSecret,
        ux_bucket: Bucket,
        network: Network,
        dgraph_cluster: DgraphCluster,
        forwarder: Optional[MetricForwarder] = None,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:

        name = "graphql-endpoint"
        super().__init__("grapl:GraphQL", name, None, opts)

        self.role = LambdaExecutionRole(name, opts=pulumi.ResourceOptions(parent=self))

        self.function = Lambda(
            name,
            args=LambdaArgs(
                execution_role=self.role,
                description=GLOBAL_LAMBDA_ZIP_TAG,
                handler="server.handler",
                runtime=aws.lambda_.Runtime.NODE_JS14D_X,
                code_path=code_path_for(name),
                package_type="Zip",
                env={
                    "MG_ALPHAS": dgraph_cluster.alpha_host_port,
                    "JWT_SECRET_ID": secret.secret.arn
                    if not LOCAL_GRAPL
                    else "JWT_SECRET_ID",  # TODO: Don't think this is
                    # actually needed in localstack anymore, provided
                    # we can properly resolve it.
                    "DEPLOYMENT_NAME": pulumi.get_stack(),
                    # TODO: This will fail in localstack becase of the
                    # URLs involved... actually, this doesn't appear
                    # to be used
                    "UX_BUCKET_URL": pulumi.Output.concat(
                        "https://", ux_bucket.bucket_regional_domain_name
                    ),
                },
                timeout=30,
                memory_size=128,
            ),
            network=network,
            forwarder=forwarder,
            opts=pulumi.ResourceOptions(parent=self),
        )

        secret.grant_read_permissions_to(self.role)
        dgraph_cluster.allow_connections_from(self.function.security_group)

        self.register_outputs({})
