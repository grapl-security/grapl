import sys

sys.path.insert(0, "..")

from typing import Mapping

import pulumi_aws as aws
from infra import config, log_levels
from infra.artifacts import ArtifactGetter
from infra.autotag import register_auto_tags
from infra.config import repository_path
from infra.docker_images import DockerImageId, DockerImageIdBuilder
from infra.grapl_stack import GraplStack
from infra.hashicorp_provider import get_nomad_provider_address
from infra.kafka import Credential, Kafka
from infra.nomad_job import NomadJob, NomadVars
from infra.observability_env_vars import get_observability_env_vars

import pulumi


def _rust_integration_container_images(
    artifacts: ArtifactGetter,
) -> Mapping[str, DockerImageId]:
    """
    Build a map of {task name -> docker image identifier}.
    """
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

    nomad_provider: pulumi.ProviderResource | None = None
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
        "graph_db": grapl_stack.graph_db,
        "aws_env_vars_for_local": grapl_stack.aws_env_vars_for_local,
        "aws_region": aws.get_region().name,
        "container_images": _rust_integration_container_images(artifacts),
        "kafka_bootstrap_servers": kafka.bootstrap_servers(),
        "kafka_consumer_group": kafka.consumer_group("integration-tests"),
        "kafka_credentials": kafka_credentials,
        "rust_log": log_levels.RUST_LOG_LEVELS,
        "observability_env_vars": get_observability_env_vars(),
        "organization_management_db": grapl_stack.organization_management_db,
        "plugin_work_queue_db": grapl_stack.plugin_work_queue_db,
        "user_auth_table": grapl_stack.user_auth_table,
        "user_session_table": grapl_stack.user_session_table,
    }

    NomadJob(
        "rust-integration-tests",
        jobspec=repository_path("nomad/rust-integration-tests.nomad"),
        vars=rust_integration_tests_job_vars,
        opts=pulumi.ResourceOptions(provider=nomad_provider),
    )


if __name__ == "__main__":
    main()
