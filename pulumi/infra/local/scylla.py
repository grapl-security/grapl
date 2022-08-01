from typing import Optional

from infra import config
from typing_extensions import Protocol, TypedDict

import pulumi


class NomadServiceScyllaDbArgs(TypedDict):
    # space delimited string host:port
    addresses: str
    username: str
    password: str


# a Pulumi resource that provides the above.
class NomadServiceScyllaResource(Protocol):
    def to_nomad_service_db_args(self) -> pulumi.Output[NomadServiceScyllaDbArgs]:
        pass


class LocalScyllaInstance(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        port: int,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:ScyllaInstance", name, None, opts)

        self.username = "cassandra"
        self.password = "cassandra"
        self.addresses = f"{config.HOST_IP_IN_NOMAD}:{port}"

    def to_nomad_service_db_args(self) -> pulumi.Output[NomadServiceScyllaDbArgs]:
        return pulumi.Output.from_input(
            NomadServiceScyllaDbArgs(
                {
                    "addresses": self.addresses,
                    "username": self.username,
                    "password": self.password,
                }
            )
        )
