from typing import Optional
from urllib.parse import urlparse

from infra.api import Api
from infra.config import DEPLOYMENT_NAME, GLOBAL_LAMBDA_ZIP_TAG, GRAPL_TEST_USER_NAME
from infra.dgraph_cluster import DgraphCluster
from infra.network import Network
from infra.secret import JWTSecret, TestUserPassword
from infra.swarm import Ec2Port

import pulumi


class E2eTestRunner(pulumi.ComponentResource):
    def __init__(
        self,
        network: Network,
        dgraph_cluster: DgraphCluster,
        api: Api,
        jwt_secret: JWTSecret,
        test_user_password: TestUserPassword,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        name = "e2e-test-runner"
        super().__init__("grapl:E2eTestRunner", name, None, opts)

        from infra.lambda_ import (
            Lambda,
            LambdaExecutionRole,
            PythonLambdaArgs,
            code_path_for,
        )

        self.role = LambdaExecutionRole(
            name,
            opts=pulumi.ResourceOptions(parent=self),
        )
        self.function = Lambda(
            name,
            args=PythonLambdaArgs(
                description=GLOBAL_LAMBDA_ZIP_TAG,
                handler="lambdex_handler.handler",
                code_path=code_path_for(name),
                env={
                    "GRAPL_LOG_LEVEL": "DEBUG",
                    "DEPLOYMENT_NAME": DEPLOYMENT_NAME,
                    "GRAPL_TEST_USER_NAME": GRAPL_TEST_USER_NAME,
                    "MG_ALPHAS": dgraph_cluster.alpha_host_port,
                    "GRAPL_API_HOST": api.invoke_url.apply(
                        lambda url: urlparse(url).netloc
                    ),
                },
                timeout=60 * 15,  # 15 minutes
                memory_size=256,
                execution_role=self.role,
            ),
            network=network,
            # graplctl expects this specific function name :(
            override_name=f"{DEPLOYMENT_NAME}-e2e-test-runner",
            opts=pulumi.ResourceOptions(parent=self),
        )

        Ec2Port("tcp", 443).allow_outbound_any_ip(self.function.security_group)

        jwt_secret.grant_read_permissions_to(self.role)
        test_user_password.grant_read_permissions_to(self.role)
        dgraph_cluster.allow_connections_from(self.function.security_group)

        self.register_outputs({})
