from typing import Optional

from infra.bucket import Bucket
from infra.config import GLOBAL_LAMBDA_ZIP_TAG, mg_alphas
from infra.dynamodb import DynamoDB
from infra.engagement_notebook import EngagementNotebook
from infra.lambda_ import Lambda, LambdaExecutionRole, PythonLambdaArgs, code_path_for
from infra.network import Network
from infra.secret import JWTSecret

import pulumi


# TODO: Rename to something like "Auth"
class EngagementEdge(pulumi.ComponentResource):
    def __init__(
        self,
        network: Network,
        secret: JWTSecret,
        ux_bucket: Bucket,
        db: DynamoDB,
        # This is optional ONLY because Localstack doesn't support
        # sagemaker :(
        notebook: Optional[EngagementNotebook] = None,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:

        name = "engagement-edge"
        super().__init__("grapl:EngagementEdge", name, None, opts)

        self.role = LambdaExecutionRole(name, opts=pulumi.ResourceOptions(parent=self))

        self.function = Lambda(
            name,
            args=PythonLambdaArgs(
                handler="lambdex_handler.handler",
                description=GLOBAL_LAMBDA_ZIP_TAG,
                code_path=code_path_for(name),
                env={
                    "GRAPL_LOG_LEVEL": "DEBUG",
                    # TODO: Not clear that this is even used.
                    "MG_ALPHAS": mg_alphas(),
                    "JWT_SECRET_ID": secret.secret.arn,
                    "USER_AUTH_TABLE": db.user_auth_table.name,
                    # TODO: Not clear that this is even used.
                    "UX_BUCKET_URL": pulumi.Output.concat(
                        "https://", ux_bucket.bucket_regional_domain_name
                    ),
                    # TODO: We *should* be passing in the name of the
                    # notebook here, rather than assuming the name is
                    # based on DEPLOYMENT_NAME.
                    "DEPLOYMENT_NAME": pulumi.get_stack(),
                },
                timeout=25,
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
