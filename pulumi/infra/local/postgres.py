from typing import Optional

import pulumi_postgresql as postgresql
from infra import config
from typing_extensions import TypedDict

import pulumi


class PostgresDbArgs(TypedDict):
    hostname: str
    port: int
    username: str
    password: str


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

    def to_nomad_vars(self) -> pulumi.Output[PostgresDbArgs]:
        return {
            "hostname": self.hostname,
            "port": self.port,
            "username": self.username,
            "password": self.password,
        }
