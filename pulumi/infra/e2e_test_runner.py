from typing import Optional

from infra.api import Api
from infra.config import DEPLOYMENT_NAME, GLOBAL_LAMBDA_ZIP_TAG, GRAPL_TEST_USER_NAME
from infra.dgraph_cluster import DgraphCluster
from infra.network import Network

import pulumi


class E2eTestRunner(pulumi.ComponentResource):
    def __init__(
        self,
        network: Network,
        dgraph_cluster: DgraphCluster,
        api: Api,
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
                execution_role=self.role,
                handler="lambdex_handler.handler",
                code_path=code_path_for(name),
                env={
                    "GRAPL_LOG_LEVEL": "INFO",
                    "DEPLOYMENT_NAME": DEPLOYMENT_NAME,
                    "GRAPL_TEST_USER_NAME": GRAPL_TEST_USER_NAME,
                    "MG_ALPHAS": dgraph_cluster.alpha_host_port,
                    "GRAPL_API_HOST": api.invoke_url,
                },
                timeout=600,
            ),
            # graplctl expects this specific function name :(
            override_name=f"{DEPLOYMENT_NAME}-e2e-test-runner]",
            network=network,
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.register_outputs({})
