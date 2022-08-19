from __future__ import annotations

from dataclasses import dataclass
from typing import Optional

from infra.local.scylla import NomadServiceScyllaDbArgs

import pulumi


@dataclass
class ProdScyllaConfigValues:
    addresses: list[str]

    def __post_init__(self) -> None:
        for addr in self.addresses:
            # TODO: assert each one is ip:port
            pass

    @classmethod
    def from_config(cls) -> ProdScyllaConfigValues:
        return cls(
            addresses=pulumi.Config().require("scylla-addresses"),
        )


class ProdScyllaInstance(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:ProdScyllaInstance", name, None, opts)

        self.username = "cassandra"
        self.password = "cassandra"
        self.addresses = ProdScyllaConfigValues.from_config().join(",")

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
