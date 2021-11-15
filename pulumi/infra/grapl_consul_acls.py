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
        )

        ui_rw_role = consul.AclRole(
            f"{name}-ui-rw",
            name="ui-read-write",
            description="Allow users to use the consul UI with write permissions",
            policies=[policies["ui-read-write"].id],
        )

        role_consul_agent = consul.AclRole(
            f"{name}-consul-agent",
            name="consul-agent",
            description="Role for consul agents, focused on nomad agents",
            policies=[policies["consul-agent"].id],
        )

        # TODO decide on expiration_times
        self.ui_read_only_token = consul.AclToken(
            f"{name}-ui-read-only",
            description="Token for full read access to the UI",
            roles=[ui_ro_role.name],
        )

        self.ui_read_write_token = consul.AclToken(
            f"{name}-ui-read-write",
            description="Token for write access to the UI",
            roles=[ui_rw_role.name],
        )

        self.default_consul_agent_token = consul.AclToken(
            f"{name}-default-consul-agent",
            description="Default token for consul agents",
            roles=[role_consul_agent.name],
        )

        # Attach ACL UI RO to the anonymous token for convenience
        consul.AclTokenRoleAttachment(
            f"{name}-attach-to-anon-token",
            role=ui_ro_role.name,
            token_id=self.ANONYMOUS_TOKEN_ID,
        )

        self.register_outputs({})
