from typing import Optional

import pulumi_aws as aws
from infra.config import (
    DEPLOYMENT_NAME,
    GLOBAL_LAMBDA_ZIP_TAG,
    grapl_api_host_port,
    grapl_graphql_host_port,
    mg_alphas,
    model_plugin_deployer_host_port,
)
from infra.network import Network

import pulumi


class E2eTestRunner(pulumi.ComponentResource):
    def __init__(
        self, network: Network, opts: Optional[pulumi.ResourceOptions] = None
    ) -> None:
        name = "e2e-test-runner"
        super().__init__("grapl:E2eTestRunner", name, None, opts)

        # Importing here avoids circular import hell between E2eTestrunner and
        # Lambda
        from infra.lambda_ import Lambda, LambdaArgs, LambdaExecutionRole, code_path_for

        self.role = LambdaExecutionRole(
            name,
            opts=pulumi.ResourceOptions(parent=self),
        )

        (
            model_plugin_deployer_host,
            model_plugin_deployer_port,
        ) = model_plugin_deployer_host_port()
        api_host, api_port = grapl_api_host_port()
        graphql_host, graphql_port = grapl_graphql_host_port()
        self.function = Lambda(
            name,
            args=LambdaArgs(
                description=GLOBAL_LAMBDA_ZIP_TAG,
                execution_role=self.role,
                runtime=aws.lambda_.Runtime.PYTHON3D7,
                handler="lambdex_handler.handler",
                code_path=code_path_for(name),
                package_type="Zip",
                env={
                    "GRAPL_LOG_LEVEL": "INFO",
                    "DEPLOYMENT_NAME": DEPLOYMENT_NAME,
                    "GRAPL_TEST_USER_NAME": f"{DEPLOYMENT_NAME}-test-user",
                    "MG_ALPHAS": mg_alphas(),
                    "GRAPL_MODEL_PLUGIN_DEPLOYER_HOST": model_plugin_deployer_host,
                    "GRAPL_MODEL_PLUGIN_DEPLOYER_PORT": f"{model_plugin_deployer_port}",
                    "GRAPL_API_HOST": api_host,
                    "GRAPL_HTTP_FRONTEND_PORT": f"{api_port}",
                    "GRAPL_GRAPHQL_HOST": graphql_host,
                    "GRAPL_GRAPHQL_PORT": f"{graphql_port}",
                },
                memory_size=128,
                timeout=600,
            ),
            opts=pulumi.ResourceOptions(parent=self),
            network=network,
        )

        self.register_outputs({})
