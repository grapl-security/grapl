import json
from pathlib import Path
from typing import Optional

import hcl2
import pulumi_consul as consul

import pulumi


class ConsulIntention(pulumi.ComponentResource):
    """
    This class takes in a directory of hcl intention config files, parses them into json format and uses that to create intentions.
    Alternatives:
    Switch the intention files into json format (we'd lose comments but :shrug:)
    Only define intentions in pulumi
    """

    def __init__(
        self,
        name: str,
        intention_directory: Path,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:ConsulIntention", name, None, opts)
        files = list(Path(intention_directory).glob("*.json"))

        for file in files:
            with open(file, "r") as f:
                intention = json.load(f)
                consul.ConfigEntry(
                    resource_name=f"{name}-{intention['Name']}",
                    kind=intention["Kind"],
                    name=intention["Name"],
                    config_json=json.dumps({"Sources": intention["Sources"]}),
                )
