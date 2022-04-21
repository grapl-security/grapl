import sys

sys.path.insert(0, "..")

import os
from typing import Mapping, Optional, cast

import pulumi_aws as aws
from infra import config
from infra.artifacts import ArtifactGetter
from infra.autotag import register_auto_tags
from infra.docker_images import DockerImageId, DockerImageIdBuilder
from infra.hashicorp_provider import get_nomad_provider_address
from infra.nomad_job import NomadJob, NomadVars
from infra.path import path_from_root

import pulumi


def _e2e_container_images(
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
        "e2e-tests": builder.build_with_tag("e2e-tests"),
    }


def _integration_container_images(
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
        "rust-integration-tests": builder.build_with_tag("rust-integration-tests"),
    }


def _integration_new_container_images(
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
        "rust-integration-tests-new": builder.build_with_tag(
            "rust-integration-tests-new"
        )
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

    e2e_test_job_vars: NomadVars = {
        "analyzer_bucket": grapl_stack.analyzer_bucket,
        "aws_env_vars_for_local": grapl_stack.aws_env_vars_for_local,
        "aws_region": aws.get_region().name,
        "container_images": _e2e_container_images(artifacts),
        # Used by graplctl to determine if it should manual-event or not
        "stack_name": grapl_stack.upstream_stack_name,
        "kafka_bootstrap_servers": grapl_stack.kafka_bootstrap_servers,
        "kafka_sasl_username": grapl_stack.kafka_e2e_sasl_username,
        "kafka_sasl_password": grapl_stack.kafka_e2e_sasl_password,
        "kafka_consumer_group_name": grapl_stack.kafka_e2e_consumer_group_name,
        "schema_properties_table_name": grapl_stack.schema_properties_table_name,
        "sysmon_log_bucket": grapl_stack.sysmon_log_bucket,
        "schema_table_name": grapl_stack.schema_table_name,
        "sysmon_generator_queue": grapl_stack.sysmon_generator_queue,
        "test_user_name": grapl_stack.test_user_name,
        "test_user_password_secret_id": grapl_stack.test_user_password_secret_id,
    }

    e2e_tests = NomadJob(
        "e2e-tests",
        jobspec=path_from_root("nomad/e2e-tests.nomad").resolve(),
        vars=e2e_test_job_vars,
        opts=pulumi.ResourceOptions(provider=nomad_provider),
    )

    integration_tests_new_job_vars: NomadVars = {
        "aws_env_vars_for_local": grapl_stack.aws_env_vars_for_local,
        "aws_region": aws.get_region().name,
        # commented out temporarily to get the artifact into cloudsmith
        # "container_images": _integration_new_container_images(artifacts),
        "kafka_bootstrap_servers": grapl_stack.kafka_bootstrap_servers,
        "pipeline_ingress_healthcheck_polling_interval_ms": grapl_stack.pipeline_ingress_healthcheck_polling_interval_ms,
        "pipeline_ingress_kafka_consumer_group_name": grapl_stack.pipeline_ingress_kafka_consumer_group_name,
        "pipeline_ingress_kafka_sasl_username": grapl_stack.pipeline_ingress_kafka_sasl_username,
        "pipeline_ingress_kafka_sasl_password": grapl_stack.pipeline_ingress_kafka_sasl_password,
    }

    integration_tests_new = NomadJob(
        "integration-tests-new",
        jobspec=path_from_root("nomad/integration-tests-new.nomad").resolve(),
        vars=integration_tests_new_job_vars,
        opts=pulumi.ResourceOptions(provider=nomad_provider),
    )

    if config.LOCAL_GRAPL:
        # We don't do integration tests in AWS yet, mostly because the current
        # Python Pants integration test setup is funky and requires an on-disk
        # Grapl repo.

        integration_test_job_vars: NomadVars = {
            "aws_env_vars_for_local": grapl_stack.aws_env_vars_for_local,
            "aws_region": aws.get_region().name,
            "container_images": _integration_container_images(artifacts),
            "docker_user": os.environ["DOCKER_USER"],
            "grapl_root": os.environ["GRAPL_ROOT"],
            "kafka_bootstrap_servers": grapl_stack.kafka_bootstrap_servers,
            "kafka_sasl_username": grapl_stack.kafka_e2e_sasl_username,
            "kafka_sasl_password": grapl_stack.kafka_e2e_sasl_password,
            "redis_endpoint": grapl_stack.redis_endpoint,
            "schema_properties_table_name": grapl_stack.schema_properties_table_name,
            "test_user_name": grapl_stack.test_user_name,
            "test_user_password_secret_id": grapl_stack.test_user_password_secret_id,
            "plugin_work_queue_db_hostname": grapl_stack.plugin_work_queue_db_hostname,
            "plugin_work_queue_db_port": grapl_stack.plugin_work_queue_db_port,
            "plugin_work_queue_db_username": grapl_stack.plugin_work_queue_db_username,
            "plugin_work_queue_db_password": grapl_stack.plugin_work_queue_db_password,
            "organization_management_db_hostname": grapl_stack.organization_management_db_hostname,
            "organization_management_db_port": grapl_stack.organization_management_db_port,
            "organization_management_db_username": grapl_stack.organization_management_db_username,
            "organization_management_db_password": grapl_stack.organization_management_db_password,
        }

        integration_tests = NomadJob(
            "integration-tests",
            jobspec=path_from_root("nomad/local/integration-tests.nomad").resolve(),
            vars=integration_test_job_vars,
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
        self.analyzer_bucket = require_str("analyzers-bucket")
        self.redis_endpoint = require_str("redis-endpoint")
        self.schema_properties_table_name = require_str("schema-properties-table")
        self.schema_table_name = require_str("schema-table")
        self.sysmon_generator_queue = require_str("sysmon-generator-queue")
        self.sysmon_log_bucket = require_str("sysmon-log-bucket")
        self.test_user_name = require_str("test-user-name")

        self.plugin_work_queue_db_hostname = require_str(
            "plugin-work-queue-db-hostname"
        )
        self.plugin_work_queue_db_port = require_str("plugin-work-queue-db-port")
        self.plugin_work_queue_db_username = require_str(
            "plugin-work-queue-db-username"
        )
        self.plugin_work_queue_db_password = require_str(
            "plugin-work-queue-db-password"
        )

        self.organization_management_db_hostname = require_str(
            "organization-management-db-hostname"
        )
        self.organization_management_db_port = require_str(
            "organization-management-db-port"
        )
        self.organization_management_db_username = require_str(
            "organization-management-db-username"
        )
        self.organization_management_db_password = require_str(
            "organization-management-db-password"
        )

        self.kafka_bootstrap_servers = require_str("kafka-bootstrap-servers")

        self.kafka_e2e_sasl_username = require_str("kafka-e2e-sasl-username")
        self.kafka_e2e_sasl_password = require_str("kafka-e2e-sasl-password")
        self.kafka_e2e_consumer_group_name = require_str(
            "kafka-e2e-consumer-group-name"
        )

        self.pipeline_ingress_healthcheck_polling_interval_ms = require_str(
            "pipeline-ingress-healthcheck-polling-interval-ms"
        )
        self.pipeline_ingress_kafka_consumer_group_name = require_str(
            "pipeline-ingress-kafka-consumer-group-name"
        )
        self.pipeline_ingress_kafka_sasl_username = require_str(
            "pipeline-ingress-kafka-sasl-username"
        )
        self.pipeline_ingress_kafka_sasl_password = require_str(
            "pipeline-ingress-kafka-sasl-password"
        )

        self.test_user_password_secret_id = require_str("test-user-password-secret-id")


if __name__ == "__main__":
    main()
