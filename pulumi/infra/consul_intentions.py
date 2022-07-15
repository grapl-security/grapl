import json
from pathlib import Path

import pulumi_consul as consul

import pulumi


class ConsulIntentions(pulumi.ComponentResource):
    """
    This class takes in a directory of json intention config files, parses them and uses them to create intentions dynamically.
    """

    def __init__(
        self,
        name: str,
        intention_directory: Path,
        opts: pulumi.ResourceOptions | None = None,
    ) -> None:
        super().__init__("grapl:ConsulIntentions", name, None, opts)
        files = list(Path(intention_directory).glob("*.json"))

        for file in files:
            with open(file) as f:
                intention = json.load(f)
                if intention["Kind"] != "service-intentions":
                    raise Exception(
                        f"file {file} is not a consul intention config per its 'Kind' value."
                    )
                elif "Sources" not in intention:
                    raise Exception(f"{file} is missing Sources stanza")
                else:
                    consul.ConfigEntry(
                        resource_name=f"{name}-{intention['Name']}-intention",
                        kind=intention["Kind"],
                        name=intention["Name"],
                        config_json=json.dumps({"Sources": intention["Sources"]}),
                        opts=pulumi.ResourceOptions.merge(
                            opts, pulumi.ResourceOptions(parent=self)
                        ),
                    )
