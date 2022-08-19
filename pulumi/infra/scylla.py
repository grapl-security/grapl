from __future__ import annotations

from dataclasses import dataclass
from typing import Optional

from typing_extensions import TypedDict

import pulumi


class NomadServiceScyllaDbArgs(TypedDict):
    # space delimited string host:port
    addresses: str
    username: str
    password: str


@dataclass
class ScyllaConfigValues:
    username: pulumi.Output[str]
    password: pulumi.Output[str]
    addresses: pulumi.Output[list[str]]

    def __post_init__(self) -> None:
        for addr in self.addresses:
            # TODO: assert each one is ip:port
            pass

    @classmethod
    def from_config(cls) -> ScyllaConfigValues:
        return cls(
            username=pulumi.Config().require("scylla-username"),
            password=pulumi.Config().require_secret("scylla-password"),
            addresses=pulumi.Config().require_object("scylla-addresses"),
        )


class ScyllaInstance(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:ProdScyllaInstance", name, None, opts)

        config_values = ScyllaConfigValues.from_config()

        self.username = config_values.username
        self.password = config_values.password
        self.addresses = config_values.addresses.apply(lambda a: ",".join(a))

    def to_nomad_scylla_args(self) -> pulumi.Output[NomadServiceScyllaDbArgs]:
        return pulumi.Output.from_input(
            NomadServiceScyllaDbArgs(
                {
                    "addresses": self.addresses,
                    "username": self.username,
                    "password": self.password,
                }
            )
        )
