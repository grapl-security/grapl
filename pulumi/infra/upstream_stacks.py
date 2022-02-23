from infra import config

import pulumi


# Not sure if this is an issue yet, but we may need to eventually look into
# making this class a singleton. Pulumi gets cranky if you try to create
# multiple stack references to the same stack in a single run.
class UpstreamStacks:
    def __init__(self) -> None:
        self.consul = pulumi.StackReference(f"grapl/consul/{config.STACK_NAME}")
        self.nomad_server = pulumi.StackReference(f"grapl/nomad/{config.STACK_NAME}")
        self.networking = pulumi.StackReference(f"grapl/networking/{config.STACK_NAME}")
        self.nomad_agents = pulumi.StackReference(
            f"grapl/nomad-agents/{config.STACK_NAME}"
        )
