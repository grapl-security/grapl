import json
from typing import Optional

import pulumi_consul as consul

import pulumi


class ConsulServiceDefault(pulumi.ComponentResource):
    """
    Create a Service Default type of Consul Config Entry.
    This is primarily for setting a non-default protocol
    """

    def __init__(
        self,
        name: str,
        service_name: str,
        protocol: str,
        additional_config_options: Optional[dict] = {},
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:ConsulServiceDefault", name, None, opts)

        consul.ConfigEntry(
            resource_name=f"{name}-{service_name}-service-defaults",
            kind="service-defaults",
            name=service_name,
            # TODO allow other things besides protocol in this config
            config_json=json.dumps({"protocol": protocol, **additional_config_options}),
            opts=pulumi.ResourceOptions.merge(
                opts, pulumi.ResourceOptions(parent=self)
            ),
        )
