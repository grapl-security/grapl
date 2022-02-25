from typing import Optional

import pulumi_postgresql as postgresql
from infra import config

import pulumi


class LocalPostgresInstance(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        port: int,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:PostgresInstance", name, None, opts)

        self.username = "postgres"
        self.password = "postgres"
        self.port = port
        self.hostname = config.HOST_IP_IN_NOMAD

        self.instance = postgresql.Database(name)
