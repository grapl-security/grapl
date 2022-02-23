import pulumi
from infra import config

class UpstreamStacks:
    def __init__(self) -> None:
        self.consul_stack = pulumi.StackReference(f"grapl/consul/{config.STACK_NAME}")
        self.nomad_server_stack = pulumi.StackReference(f"grapl/nomad/{config.STACK_NAME}")
        self.networking_stack = pulumi.StackReference(
            f"grapl/networking/{config.STACK_NAME}"
        )
        self.nomad_agents_stack = pulumi.StackReference(
            f"grapl/nomad-agents/{config.STACK_NAME}"
        )