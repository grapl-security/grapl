import os
import sys
from pathlib import Path

sys.path.insert(0, "..")

from typing import Mapping, Optional, cast

import pulumi_aws as aws
import pulumi_nomad as nomad
from infra import config
from infra.autotag import register_auto_tags
from infra.docker_images import DockerImageId, DockerImageIdBuilder, version_tag
from infra.nomad_job import NomadJob, NomadVars
from infra.quiet_docker_build_output import quiet_docker_output

import pulumi


def _container_images(
    artifacts: Mapping[str, str], require_artifact: bool = False
) -> Mapping[str, DockerImageId]:
    """
    Build a map of {task name -> docker image identifier}.
    """
    img_id_builder = DockerImageIdBuilder(config.container_repository())

    def build_img_id_with_tag(image_name: str) -> DockerImageId:
        """
        A shortcut to grab the version tag from the artifacts map and build a
        DockerImageId out of it
        """
        tag = version_tag(image_name, artifacts, require_artifact)
        return img_id_builder.build(image_name=image_name, tag=tag)

    return {
        "e2e-tests": build_img_id_with_tag("e2e-tests"),
        "python-integration-tests": build_img_id_with_tag("python-integration-tests"),
        "rust-integration-tests": build_img_id_with_tag("rust-integration-tests"),
    }


def main() -> None:
    ##### Preamble
    stack_name = pulumi.get_stack()

    pulumi_config = pulumi.Config()
    artifacts = pulumi_config.get_object("artifacts") or {}
    container_images = _container_images(
        artifacts, require_artifact=(not config.LOCAL_GRAPL)
    )

    quiet_docker_output()

    # These tags will be added to all provisioned infrastructure
    # objects.
    register_auto_tags({"grapl deployment": stack_name})

    if not config.LOCAL_GRAPL:
        # TODO twunderlich: DRY this up

        # Set the nomad address. This can be either set as nomad:address in the config to support ssm port forwarding or
        # taken from the nomad stack
        nomad_config = pulumi.Config("nomad")
        nomad_override_address = nomad_config.get("address")
        # We prefer nomad:address to support overriding in the case of ssm port forwarding
        nomad_server_stack = pulumi.StackReference(f"grapl/nomad/{stack_name}")
        nomad_address = nomad_override_address or nomad_server_stack.require_output(
            "address"
        )
        nomad_provider = nomad.Provider("nomad-aws", address=nomad_address)

    ##### Business Logic

    grapl_stack = GraplStack(stack_name)

    access_key = aws.config.access_key
    secret_key = aws.config.secret_key

    e2e_test_job_vars: NomadVars = {
        "analyzer_bucket": grapl_stack.analyzer_bucket,
        "aws_access_key_id": access_key,
        "aws_access_key_secret": secret_key,
        "_aws_endpoint": grapl_stack.aws_endpoint,
        "aws_region": aws.get_region().name,
        "container_images": container_images,
        "deployment_name": grapl_stack.deployment_name,
        "schema_properties_table_name": grapl_stack.schema_properties_table_name,
        "sysmon_log_bucket": grapl_stack.sysmon_log_bucket,
        "schema_table_name": grapl_stack.schema_table_name,
        "sysmon_generator_queue": grapl_stack.sysmon_generator_queue,
        "test_user_name": grapl_stack.test_user_name,
    }

    e2e_tests = NomadJob(
        "e2e-tests",
        jobspec=Path("../../nomad/local/e2e-tests.nomad").resolve(),
        vars=e2e_test_job_vars,
    )

    if config.LOCAL_GRAPL:
        # We don't do integration tests in AWS yet, mostly because the current
        # Python Pants integration test setup is funky and requires an on-disk
        # Grapl repo.

        integration_test_job_vars: NomadVars = {
            "aws_access_key_id": access_key,
            "aws_access_key_secret": secret_key,
            "_aws_endpoint": grapl_stack.aws_endpoint,
            "aws_region": aws.get_region().name,
            "container_images": container_images,
            "deployment_name": grapl_stack.deployment_name,
            "docker_user": os.environ["DOCKER_USER"],
            "grapl_root": os.environ["GRAPL_ROOT"],
            "_kafka_endpoint": grapl_stack.kafka_endpoint,
            "_redis_endpoint": grapl_stack.redis_endpoint,
            "schema_properties_table_name": grapl_stack.schema_properties_table_name,
            "test_user_name": grapl_stack.test_user_name,
        }

        integration_tests = NomadJob(
            "integration-tests",
            jobspec=Path("../../nomad/local/integration-tests.nomad").resolve(),
            vars=integration_test_job_vars,
        )


class GraplStack:
    def __init__(self, stack_name: str) -> None:
        ref_name = "local-grapl" if config.LOCAL_GRAPL else f"grapl/grapl/{stack_name}"
        ref = pulumi.StackReference(ref_name)

        def require_str(key: str) -> str:
            return cast(str, ref.require_output(key))

        # Only specified if LOCAL_GRAPL
        self.aws_endpoint = cast(Optional[str], ref.get_output("aws-endpoint"))

        self.analyzer_bucket = require_str("analyzers-bucket")
        self.deployment_name = require_str("deployment-name")
        self.kafka_endpoint = require_str("kafka-endpoint")
        self.redis_endpoint = require_str("redis-endpoint")
        self.schema_properties_table_name = require_str("schema-properties-table")
        self.schema_table_name = require_str("schema-table")
        self.sysmon_generator_queue = require_str("sysmon-generator-queue")
        self.sysmon_log_bucket = require_str("sysmon-log-bucket")
        self.test_user_name = require_str("test-user-name")


if __name__ == "__main__":
    main()
