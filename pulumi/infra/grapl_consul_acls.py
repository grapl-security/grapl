from typing import Optional

import pulumi_consul as consul
from infra.consul_acl_policies import ConsulAclPolicies

import pulumi


class GraplConsulAcls(pulumi.ComponentResource):

    ANONYMOUS_TOKEN_ID = "00000000-0000-0000-0000-000000000002"

    def __init__(
        self,
        name: str,
        policies: ConsulAclPolicies,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:GraplConsulAcls", "name", None, opts)

        # Create roles
        ui_ro_role = consul.AclRole(
            "ui-ro",
            name="ui-read-only",
            description="Allow users to use the consul UI with read-only permissions",
            policies=[policies["ui-read-only"].name],
        )

        ui_rw_role = consul.AclRole(
            "ui-rw",
            name="ui-read-write",
            description="Allow users to use the consul UI with write permissions",
            policies=[policies["ui-read-write"].name],
        )

        role_consul_agent = consul.AclRole(
            "consul-agent",
            name="consul-agent",
            description="Role for consul agents, focused on nomad agents",
            policies=[policies["consul-agent"].name],
        )

        # TODO decide on expiration_times
        self.ui_read_only_token = consul.AclToken(
            "ui-read-only",
            description="Token for full read access to the UI",
            roles=[ui_ro_role.name],
        )

        self.ui_read_write_token = consul.AclToken(
            "ui-read-write",
            description="Token for write access to the UI",
            roles=[ui_rw_role.name],
        )

        self.default_consul_agent_token = consul.AclToken(
            "default-consul-agent",
            description="Default token for consul agents",
            roles=[role_consul_agent.name],
        )

        # Attach ACL UI RO to the anonymous token for convenience
        consul.AclTokenRoleAttachment(
            "attach-to-anon-token",
            role=ui_ro_role.name,
            token_id=self.ANONYMOUS_TOKEN_ID,
        )
