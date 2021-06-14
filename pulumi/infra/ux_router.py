from typing import Optional

from infra.bucket import Bucket
from infra.config import GLOBAL_LAMBDA_ZIP_TAG, configurable_envvars
from infra.ec2 import Ec2Port
from infra.lambda_ import Lambda, LambdaExecutionRole, PythonLambdaArgs, code_path_for
from infra.metric_forwarder import MetricForwarder
from infra.network import Network
from infra.secret import JWTSecret

import pulumi


class UxRouter(pulumi.ComponentResource):
    def __init__(
        self,
        network: Network,
        secret: JWTSecret,
        ux_bucket: Bucket,
        forwarder: MetricForwarder,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:

        name = "ux-router"
        super().__init__("grapl:UXRouter", name, None, opts)

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
                    "UX_BUCKET_NAME": ux_bucket.bucket,
                },
                timeout=5,
                memory_size=128,
            ),
            # TODO: I don't think we need a network, because I don't
            # think this needs access to anything in the VPC itself.
            network=network,
            opts=pulumi.ResourceOptions(parent=self),
        )

        Ec2Port("tcp", 443).allow_outbound_any_ip(self.function.security_group)

        forwarder.subscribe_to_log_group(name, self.function.log_group)

        ux_bucket.grant_read_permission_to(self.role)
        secret.grant_read_permissions_to(self.role)

        self.register_outputs({})
