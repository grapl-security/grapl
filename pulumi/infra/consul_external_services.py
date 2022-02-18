from typing import Optional

import pulumi_consul as consul

import pulumi


class ConsulExternalServices(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:ConsulExternalServices", name, None, opts)
        child_opts = pulumi.ResourceOptions(parent=self)

        address = (
            # TODO if-else
            # when Consul runs locally, it accesses Localstack over the host
            # network - so no host.docker.internal or attr.network.unique shenanigans
            # "localhost"
            "127.0.0.1"
        )
        aws_node = consul.Node(
            f"{name}-aws-node", address=address, name="aws-node", opts=child_opts
        )

        s3_port = (
            # TODO if-else
            "4566"
        )
        s3_service = consul.Service(
            f"{name}-s3-service",
            node=aws_node.name,
            name="s3",
            port=s3_port,
            opts=child_opts,
        )
