import sys

sys.path.insert(0, "..")

from typing import Mapping, Optional, cast

import pulumi_aws as aws
from infra import config, log_levels
from infra.artifacts import ArtifactGetter
from infra.autotag import register_auto_tags
from infra.docker_images import DockerImageId, DockerImageIdBuilder
from infra.hashicorp_provider import get_nomad_provider_address
from infra.kafka import Credential, Kafka
from infra.nomad_job import NomadJob, NomadVars
from infra.nomad_service_postgres import NomadServicePostgresDbArgs
from infra.path import path_from_root

import pulumi


def _rust_integration_container_images(
    artifacts: ArtifactGetter,
) -> Mapping[str, DockerImageId]:
    """Build a map of {task name -> docker image identifier}."""
    builder = DockerImageIdBuilder(
        container_repository=config.container_repository(),
        artifacts=artifacts,
    )

    return {
        "rust-integration-tests": builder.build_with_tag("rust-integration-tests"),
    }


def main() -> None:
    ##### Preamble
    stack_name = config.STACK_NAME

    pulumi_config = pulumi.Config()
    artifacts = ArtifactGetter.from_config(pulumi_config)

    # These tags will be added to all provisioned infrastructure
    # objects.
    register_auto_tags(
        {"pulumi:project": pulumi.get_project(), "pulumi:stack": stack_name}
    )

    nomad_provider: Optional[pulumi.ProviderResource] = None
    if not config.LOCAL_GRAPL:
        nomad_server_stack = pulumi.StackReference(f"grapl/nomad/{stack_name}")
        nomad_provider = get_nomad_provider_address(nomad_server_stack)

    ##### Business Logic
    grapl_stack = GraplStack(stack_name)

    kafka = Kafka(
        "kafka",
        confluent_environment_name=pulumi_config.require("confluent-environment-name"),
        create_local_topics=False,
    )

    kafka_credentials = kafka.service_credentials("integration-tests").apply(
        Credential.to_nomad_service_creds
    )

    rust_integration_tests_job_vars: NomadVars = {
        "aws_env_vars_for_local": grapl_stack.aws_env_vars_for_local,
        "aws_region": aws.get_region().name,
        "container_images": _rust_integration_container_images(artifacts),
        "dns_server": config.CONSUL_DNS_IP,
        "kafka_bootstrap_servers": kafka.bootstrap_servers(),
        "kafka_consumer_group": kafka.consumer_group("integration-tests"),
        "kafka_credentials": kafka_credentials,
        "rust_log": log_levels.RUST_LOG_LEVELS,
        "organization_management_db": grapl_stack.organization_management_db,
        "plugin_work_queue_db": grapl_stack.plugin_work_queue_db,
    }

    rust_integration_tests = NomadJob(
        "rust-integration-tests",
        jobspec=path_from_root("nomad/rust-integration-tests.nomad").resolve(),
        vars=rust_integration_tests_job_vars,
        opts=pulumi.ResourceOptions(provider=nomad_provider),
    )


class GraplStack:
    def __init__(self, stack_name: str) -> None:
        self.upstream_stack_name = (
            "local-grapl" if config.LOCAL_GRAPL else f"grapl/grapl/{stack_name}"
        )
        ref = pulumi.StackReference(self.upstream_stack_name)

        def require_str(key: str) -> str:
            return cast(str, ref.require_output(key))

        # FIXME: audit these, they're not all required for rust integration tests
        self.aws_env_vars_for_local = require_str("aws-env-vars-for-local")
        self.analyzer_bucket = require_str("analyzers-bucket")
        self.redis_endpoint = require_str("redis-endpoint")
        self.schema_properties_table_name = require_str("schema-properties-table")
        self.schema_table_name = require_str("schema-table")
        self.sysmon_log_bucket = require_str("sysmon-log-bucket")
        self.test_user_name = require_str("test-user-name")

        self.plugin_work_queue_db = cast(
            NomadServicePostgresDbArgs, ref.require_output("plugin-work-queue-db")
        )

        self.organization_management_db = cast(
            NomadServicePostgresDbArgs, ref.require_output("organization-management-db")
        )

        self.test_user_password_secret_id = require_str("test-user-password-secret-id")


if __name__ == "__main__":
    main()
