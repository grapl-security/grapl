from typing import Optional

import pulumi_postgresql as postgresql

import pulumi


class LocalPostgresInstance(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:PostgresInstance", name, None, opts)

        self.username = "postgres"
        self.password = "postgres"
        self.port = 5432

        self.instance = postgresql.Database(name)
