import json
from pathlib import Path
from typing import Optional

import hcl2
import pulumi_consul as consul

import pulumi


class ConsulConfig(pulumi.ComponentResource):
    """
    Consul config entries
    """

    def __init__(
        self,
        name: str,
        hcl_file: Path,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:ConsulConfig", name, None, opts)

        with open(hcl_file, "r") as file:
            dict = hcl2.load(file)

            consul.ConfigEntry(
                resource_name=f"{name}-proxy-defaults",
                kind="proxy-defaults",
                name="global",
                config_json=json.dumps(dict),
                opts=pulumi.ResourceOptions.merge(
                    opts, pulumi.ResourceOptions(parent=self)
                ),
            )
