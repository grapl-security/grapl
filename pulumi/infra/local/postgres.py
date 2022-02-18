from typing import Optional

import pulumi_postgresql as postgresql

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
        self.hostname = "${attr.unique.network.ip-address}"

        self.instance = postgresql.Database(name)
