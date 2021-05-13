from typing import Optional
from urllib.parse import urlparse

<<<<<<< HEAD
from infra.api import Api
from infra.config import DEPLOYMENT_NAME, GLOBAL_LAMBDA_ZIP_TAG, GRAPL_TEST_USER_NAME
from infra.dgraph_cluster import DgraphCluster
=======
import pulumi_aws as aws
from infra.config import (
    DEPLOYMENT_NAME,
    GLOBAL_LAMBDA_ZIP_TAG,
    grapl_api_host_port,
    grapl_graphql_host_port,
    mg_alphas,
    model_plugin_deployer_host_port,
)
>>>>>>> make provisioner lambda run in localstack and e2e tests run in lambda
from infra.network import Network
from infra.secret import JWTSecret, TestUserPassword
from infra.swarm import Ec2Port

from infra.config import DEPLOYMENT_NAME, GLOBAL_LAMBDA_ZIP_TAG

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
<<<<<<< HEAD
=======

        (
            model_plugin_deployer_host,
            model_plugin_deployer_port,
        ) = model_plugin_deployer_host_port()
        api_host, api_port = grapl_api_host_port()
        graphql_host, graphql_port = grapl_graphql_host_port()
>>>>>>> make provisioner lambda run in localstack and e2e tests run in lambda
        self.function = Lambda(
            name,
            args=PythonLambdaArgs(
                description=GLOBAL_LAMBDA_ZIP_TAG,
                handler="lambdex_handler.handler",
                code_path=code_path_for(name),
                env={
                    "GRAPL_LOG_LEVEL": "DEBUG",
                    "DEPLOYMENT_NAME": DEPLOYMENT_NAME,
<<<<<<< HEAD
                    "GRAPL_TEST_USER_NAME": GRAPL_TEST_USER_NAME,
                    "MG_ALPHAS": dgraph_cluster.alpha_host_port,
                    "GRAPL_API_HOST": api.invoke_url.apply(
                        lambda url: urlparse(url).netloc
                    ),
=======
                    "GRAPL_TEST_USER_NAME": f"{DEPLOYMENT_NAME}-test-user",
                    "MG_ALPHAS": mg_alphas(),
                    "GRAPL_MODEL_PLUGIN_DEPLOYER_HOST": model_plugin_deployer_host,
                    "GRAPL_MODEL_PLUGIN_DEPLOYER_PORT": f"{model_plugin_deployer_port}",
                    "GRAPL_API_HOST": api_host,
                    "GRAPL_HTTP_FRONTEND_PORT": f"{api_port}",
                    "GRAPL_GRAPHQL_HOST": graphql_host,
                    "GRAPL_GRAPHQL_PORT": f"{graphql_port}",
>>>>>>> make provisioner lambda run in localstack and e2e tests run in lambda
                },
                timeout=60 * 5,  # 5 minutes
                memory_size=256,
                execution_role=self.role,
            ),
            network=network,
            # graplctl expects this specific function name :(
            override_name=f"{DEPLOYMENT_NAME}-e2e-test-runner",
            opts=pulumi.ResourceOptions(parent=self),
            network=network,
        )

        Ec2Port("tcp", 443).allow_outbound_any_ip(self.function.security_group)

        jwt_secret.grant_read_permissions_to(self.role)
        test_user_password.grant_read_permissions_to(self.role)
        dgraph_cluster.allow_connections_from(self.function.security_group)

        self.register_outputs({})
