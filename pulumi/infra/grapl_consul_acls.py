from typing import Dict, Optional

import pulumi_consul as consul

import pulumi


class GraplConsulAcls(pulumi.ComponentResource):

    ANONYMOUS_TOKEN_ID = "00000000-0000-0000-0000-000000000002"

    def __init__(
        self,
        name: str,
        policies: Dict[str, consul.AclPolicy],
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:GraplConsulAcls", "name", None, opts)

        # Create roles
        ui_ro_role = consul.AclRole(
            f"{name}-ui-ro",
            name="ui-read-only",
            description="Allow users to use the consul UI with read-only permissions",
            policies=[policies["ui-read-only"].id],
            opts=pulumi.ResourceOptions(parent=self),
        )

        ui_rw_role = consul.AclRole(
            f"{name}-ui-rw",
            name="ui-read-write",
            description="Allow users to use the consul UI with write permissions",
            policies=[policies["ui-read-write"].id],
            opts=pulumi.ResourceOptions(parent=self),
        )

        consul_agent_role = consul.AclRole(
            f"{name}-consul-agent",
            name="consul-agent",
            description="Role for consul agents, focused on nomad agents",
            policies=[policies["consul-agent"].id],
            opts=pulumi.ResourceOptions(parent=self),
        )

        # TODO decide on expiration_times
        self.ui_read_only_token = consul.AclToken(
            f"{name}-ui-read-only",
            description="Token for full read access to the UI",
            roles=[ui_ro_role.name],
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.ui_read_write_token = consul.AclToken(
            f"{name}-ui-read-write",
            description="Token for write access to the UI",
            roles=[ui_rw_role.name],
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.default_consul_agent_token = consul.AclToken(
            f"{name}-default-consul-agent",
            description="Default token for consul agents",
            roles=[consul_agent_role.name],
            opts=pulumi.ResourceOptions(parent=self),
        )

        # Attach ACL UI RO to the anonymous token for convenience
        consul.AclTokenRoleAttachment(
            f"{name}-attach-ui-ro-to-anon-token",
            role=ui_ro_role.name,
            token_id=self.ANONYMOUS_TOKEN_ID,
            opts=pulumi.ResourceOptions(parent=self),
        )

        # Ugly Hack Warning: Since there's no api to assign a token to a consul agent, we assign to the anonymous token
        # for the moment
        consul.AclTokenRoleAttachment(
            f"{name}-attach-consul-agent-to-anon-token",
            role=consul_agent_role.name,
            token_id=self.ANONYMOUS_TOKEN_ID,
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.register_outputs({})
