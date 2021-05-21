from typing import Optional

from infra.config import DEPLOYMENT_NAME, GLOBAL_LAMBDA_ZIP_TAG
from infra.dgraph_cluster import DgraphCluster
from infra.dynamodb import DynamoDB
from infra.lambda_ import Lambda, LambdaExecutionRole, PythonLambdaArgs, code_path_for
from infra.network import Network
from infra.secret import JWTSecret

import pulumi


class Provisioner(pulumi.ComponentResource):
    def __init__(
        self,
        network: Network,
        secret: JWTSecret,
        dynamodb: DynamoDB,
        dgraph_cluster: DgraphCluster,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:

        name = "provisioner"
        super().__init__("grapl:Provisioner", name, None, opts)

        self.role = LambdaExecutionRole(name, opts=pulumi.ResourceOptions(parent=self))

        self.function = Lambda(
            name,
            args=PythonLambdaArgs(
                handler="lambdex_handler.handler",
                description=GLOBAL_LAMBDA_ZIP_TAG,
                code_path=code_path_for(name),
                env={
                    # TODO: this is mostly copy pasted, figure out what we actually need
                    "GRAPL_LOG_LEVEL": "DEBUG",
                    "DEPLOYMENT_NAME": DEPLOYMENT_NAME,
                    "MG_ALPHAS": dgraph_cluster.alpha_host_port,
                    "GRAPL_TEST_USER_NAME": f"{DEPLOYMENT_NAME}-grapl-test-user",
                },
                timeout=600,
                memory_size=256,
                execution_role=self.role,
            ),
            network=network,
            # graplctl currently expects a very specific function name.
            # in the future we should just have it pull from pulumi outputs.
            override_name=f"{DEPLOYMENT_NAME}-Provisioner-Handler",
            opts=pulumi.ResourceOptions(parent=self),
        )

        secret.grant_read_permissions_to(self.role)
        dynamodb.user_auth_table.grant_read_permissions_to(self.role)
        dgraph_cluster.allow_connections_from(self.function.security_group)

        self.register_outputs({})
