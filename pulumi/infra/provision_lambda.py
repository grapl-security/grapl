from typing import Optional

from infra.bucket import Bucket
from infra.config import DEPLOYMENT_NAME, GLOBAL_LAMBDA_ZIP_TAG, mg_alphas
from infra.dynamodb import DynamoDB
from infra.engagement_notebook import EngagementNotebook
from infra.lambda_ import Lambda, LambdaExecutionRole, PythonLambdaArgs, code_path_for
from infra.network import Network
from infra.secret import JWTSecret

import pulumi


class Provisioner(pulumi.ComponentResource):
    def __init__(
        self,
        network: Network,
        secret: JWTSecret,
        db: DynamoDB,
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
                    "GRAPL_LOG_LEVEL": "DEBUG",
                    "DEPLOYMENT_NAME": pulumi.get_stack(),
                    # TODO: Not clear that this is even used.
                    "MG_ALPHAS": mg_alphas(),
                    "GRAPL_TEST_USER_NAME": f"{DEPLOYMENT_NAME}-grapl-test-user",
                },
                timeout=600,
                memory_size=256,
                execution_role=self.role,
            ),
            network=network,
            # TODO: Forwarder????
            opts=pulumi.ResourceOptions(parent=self),
        )

        # TODO: Original infrastructure code allowed access to DGraph,
        # but it's not clear this is even necessary.

        if notebook:
            notebook.grant_presigned_url_permissions_to(self.role)

        secret.grant_read_permissions_to(self.role)
        db.user_auth_table.grant_read_permissions_to(self.role)

        self.register_outputs({})
