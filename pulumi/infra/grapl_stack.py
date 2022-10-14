from typing import cast

from infra import config
from infra.nomad_service_postgres import NomadServicePostgresDbArgs
from infra.scylla import NomadServiceScyllaDbArgs

import pulumi


class GraplStack:
    """An object-oriented abstraction around accessing values from a
    `grapl` project `StackReference`.

    Useful for accessing infrastructure information in integration
    test projects.

    """

    def __init__(self, stack_name: str) -> None:
        self.upstream_stack_name = (
            "local-grapl" if config.LOCAL_GRAPL else f"grapl/grapl/{stack_name}"
        )
        ref = pulumi.StackReference(self.upstream_stack_name)

        def require_str(key: str) -> str:
            return cast(str, ref.require_output(key))

        self.aws_env_vars_for_local = require_str("aws-env-vars-for-local")
        self.schema_properties_table_name = require_str("schema-properties-table")
        self.schema_table_name = require_str("schema-table")
        self.test_user_name = require_str("test-user-name")

        self.plugin_work_queue_db = cast(
            NomadServicePostgresDbArgs, ref.require_output("plugin-work-queue-db")
        )

        self.organization_management_db = cast(
            NomadServicePostgresDbArgs, ref.require_output("organization-management-db")
        )

        self.test_user_password_secret_id = require_str("test-user-password-secret-id")

        self.user_auth_table = require_str("user-auth-table")
        self.user_session_table = require_str("user-session-table")

        self.graph_db = cast(NomadServiceScyllaDbArgs, ref.require_output("graph-db"))
