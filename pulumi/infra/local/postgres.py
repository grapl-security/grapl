from typing import Optional

import pulumi_postgresql as postgresql
from infra import config
from infra.nomad_job import NomadServicePostgresDbArgs

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

    def to_nomad_service_db_args(self) -> pulumi.Output[NomadServicePostgresDbArgs]:
        return pulumi.Output.from_input(
            NomadServicePostgresDbArgs(
                {
                    "hostname": self.hostname,
                    "port": self.port,
                    "username": self.username,
                    "password": self.password,
                }
            )
        )
