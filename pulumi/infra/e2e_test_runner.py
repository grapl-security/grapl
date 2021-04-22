from typing import Optional

import pulumi_aws as aws
from infra.config import DEPLOYMENT_NAME, GLOBAL_LAMBDA_ZIP_TAG, mg_alphas
from infra.network import Network

import pulumi


class E2eTestRunner(pulumi.ComponentResource):
    def __init__(self, network: Network, opts: Optional[pulumi.ResourceOptions] = None) -> None:
        name = "e2e-test-runner"
        super().__init__("grapl:E2eTestRunner", name, None, opts)

        # Importing here avoids circular import hell between E2eTestrunner and
        # Lambda
        from infra.lambda_ import Lambda, LambdaArgs, LambdaExecutionRole, code_path_for

        self.role = LambdaExecutionRole(
            name,
            opts=pulumi.ResourceOptions(parent=self),
        )

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
                    "GRAPL_DEPLOYMENT_NAME": DEPLOYMENT_NAME,
                    "GRAPL_TEST_USER_NAME": f"{DEPLOYMENT_NAME}-test-user",
                    "MG_ALPHAS": mg_alphas(),
                },
                memory_size=128,
                timeout=600,
            ),
            opts=pulumi.ResourceOptions(parent=self),
            network=network,
        )

        self.register_outputs({})
