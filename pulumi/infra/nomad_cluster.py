from typing import Optional

from infra.network import Network
from infra.nomad_server_fleet import NomadServerFleet

import pulumi


class NomadCluster(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        network: Network,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:NomadCluster", name=name, props=None, opts=opts)
        child_opts = pulumi.ResourceOptions(parent=self)

        self.nomad_server_fleet = NomadServerFleet(
            "nomad-server-fleet",
            network=network,
            internal_service_ports=(),
            opts=child_opts,
        )

        # Theoretically in the future, we'll also have a `self.nomad_client_fleet =`
