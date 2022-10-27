import sys

sys.path.insert(0, "..")

import os
from typing import Mapping

import pulumi_aws as aws
from infra import config
from infra.artifacts import ArtifactGetter
from infra.autotag import register_auto_tags
from infra.config import repository_path
from infra.docker_images import (
    DockerImageId,
    DockerImageIdBuilder,
    container_versions_from_container_ids,
)
from infra.grapl_stack import GraplStack
from infra.hashicorp_provider import get_nomad_provider_address
from infra.kafka import Kafka
from infra.nomad_job import NomadJob, NomadVars

import pulumi


def _python_integration_container_images(
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
        "python-integration-tests": builder.build_with_tag("python-integration-tests"),
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

    integration_tests_kafka_credentials = kafka.service_credentials("integration-tests")

    if config.LOCAL_GRAPL:
        # We don't do Python integration tests in AWS yet, mostly because the
        # current Python Pants integration test setup is funky and requires an
        # on-disk Grapl repo.
        # FIXME: make python integration tests work in AWS.

        container_images = _python_integration_container_images(artifacts)

        python_integration_test_job_vars: NomadVars = {
            "aws_env_vars_for_local": grapl_stack.aws_env_vars_for_local,
            "aws_region": aws.get_region().name,
            "container_images": container_images,
            "container_versions": container_versions_from_container_ids(
                container_images
            ),
            "docker_user": os.environ["DOCKER_USER"],
            "grapl_root": os.environ["GRAPL_ROOT"],
            "schema_properties_table_name": grapl_stack.schema_properties_table_name,
            "test_user_name": grapl_stack.test_user_name,
            "test_user_password_secret_id": grapl_stack.test_user_password_secret_id,
        }

        NomadJob(
            "python-integration-tests",
            jobspec=repository_path("nomad/local/python-integration-tests.nomad"),
            vars=python_integration_test_job_vars,
            opts=pulumi.ResourceOptions(provider=nomad_provider),
        )


if __name__ == "__main__":
    main()
