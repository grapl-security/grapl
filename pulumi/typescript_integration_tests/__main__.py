import sys

sys.path.insert(0, "..")

from typing import Mapping, cast

import pulumi_aws as aws
from infra import config
from infra.artifacts import ArtifactGetter
from infra.autotag import register_auto_tags
from infra.config import repository_path
from infra.docker_images import DockerImageId, DockerImageIdBuilder
from infra.hashicorp_provider import get_nomad_provider_address
from infra.kafka import Credential, Kafka
from infra.nomad_job import NomadJob, NomadVars

import pulumi


def _typescript_integration_container_images(
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
        "typescript-integration-tests": builder.build_with_tag(
            "typescript-integration-tests"
        ),
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

    typescript_integration_tests_job_vars: NomadVars = {
        "aws_env_vars_for_local": grapl_stack.aws_env_vars_for_local,
        "aws_region": aws.get_region().name,
        "container_images": _typescript_integration_container_images(artifacts),
        "test_user_name": grapl_stack.test_user_name,
        # "test_user_password_secret_id": grapl_stack.test_user_password_secret_id,
    }

    typescript_integration_tests = NomadJob(
        "typescript-integration-tests",
        jobspec=repository_path("nomad/typescript-integration-tests.nomad"),
        vars=typescript_integration_tests_job_vars,
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

        self.aws_env_vars_for_local = require_str("aws-env-vars-for-local")
        self.test_user_name = require_str("test-user-name")

        # self.plugin_work_queue_db = cast(
        #     NomadServicePostgresDbArgs, ref.require_output("plugin-work-queue-db")
        # )

        self.test_user_password_secret_id = require_str("test-user-password-secret-id")


if __name__ == "__main__":
    main()
