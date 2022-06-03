from typing import Protocol

from typing_extensions import TypedDict

import pulumi


class NomadServicePostgresDbArgs(TypedDict):
    hostname: str
    port: int
    username: str
    password: str


# a Pulumi resource that provides the above.
class NomadServicePostgresResource(Protocol):
    def to_nomad_service_db_args(self) -> pulumi.Output[NomadServicePostgresDbArgs]:
        pass
