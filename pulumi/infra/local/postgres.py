from typing import Optional

import pulumi_postgresql as postgresql

import pulumi


class LocalPostgresInstance(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        port: int,
        address: Optional[str] = "LOCAL_GRAPL_REPLACE_IP",
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:PostgresInstance", name, None, opts)

        self.instance = postgresql.Database(name)
        self.instance.address = address
        self.instance.username = "postgres"
        self.instance.password = "postgres"
        self.instance.port = port
