from typing import Optional

from infra import dynamodb
from infra.config import DEPLOYMENT_NAME
from infra.dgraph_cluster import DgraphCluster
from infra.dynamodb import DynamoDB
from infra.lambda_ import Lambda, LambdaExecutionRole, LambdaResolver, PythonLambdaArgs
from infra.network import Network
from infra.secret import TestUserPassword
from infra.swarm import Ec2Port

import pulumi


class Provisioner(pulumi.ComponentResource):
    def __init__(
        self,
        network: Network,
        test_user_password: TestUserPassword,
        db: DynamoDB,
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
                code=LambdaResolver.resolve(name),
                env={
                    "GRAPL_LOG_LEVEL": "DEBUG",
                    "DEPLOYMENT_NAME": DEPLOYMENT_NAME,
                    "MG_ALPHAS": dgraph_cluster.alpha_host_port,
                    "GRAPL_TEST_USER_NAME": f"{DEPLOYMENT_NAME}-grapl-test-user",
                },
                timeout=60 * 15,  # 15 minutes
                memory_size=256,
                execution_role=self.role,
            ),
            network=network,
            opts=pulumi.ResourceOptions(parent=self),
        )

        Ec2Port("tcp", 443).allow_outbound_any_ip(self.function.security_group)

        test_user_password.grant_read_permissions_to(self.role)

        dynamodb.grant_read_write_on_tables(
            self.role, [db.user_auth_table, db.schema_table, db.schema_properties_table]
        )

        dgraph_cluster.allow_connections_from(self.function.security_group)

        self.register_outputs({})
