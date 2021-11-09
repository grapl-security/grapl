import os
import sys
from pathlib import Path

sys.path.insert(0, "..")

import pulumi_aws as aws
import pulumi_nomad as nomad
from infra import config
from infra.autotag import register_auto_tags
from infra.nomad_job import NomadJob, NomadVars
from infra.quiet_docker_build_output import quiet_docker_output

import pulumi


def stackname_sans_prefix() -> str:
    real_stackname = pulumi.get_stack()
    # If local-grapl, no orgs in play
    if config.LOCAL_GRAPL:
        return real_stackname

    prefix = "grapl/grapl"
    split = real_stackname.split(prefix)
    assert (
        len(split) == 2
    ), f"Expected a stack prefix of {prefix}, found {real_stackname}"
    return split[1]


class GraplStack:
    def __init__(self, stack_name: str) -> None:
        ref_name = "local-grapl" if config.LOCAL_GRAPL else f"grapl/grapl/{stack_name}"
        ref = pulumi.StackReference(ref_name)
        output = ref.require_output  # just an alias

        self.analyzer_bucket = output("analyzers-bucket")
        self.aws_endpoint = output("aws-endpoint")
        self.deployment_name = output("deployment-name")

        self.kafka_endpoint = output("kafka-endpoint")
        self.redis_endpoint = output("redis-endpoint")
        self.schema_properties_table_name = output("schema-properties-table")
        self.schema_table_name = output("schema-table")
        self.sysmon_generator_queue = output("sysmon-generator-queue")
        self.sysmon_log_bucket = output("sysmon-log-bucket")
        self.test_user_name = output("test-user-name")


def main() -> None:
    ##### Preamble

    stack_name = stackname_sans_prefix()

    quiet_docker_output()

    # These tags will be added to all provisioned infrastructure
    # objects.
    register_auto_tags({"grapl deployment": stack_name})

    if not config.LOCAL_GRAPL:
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

    ##### Actual Logic

    grapl_stack = GraplStack(stack_name)

    assert aws.config.access_key, "Appease typechecker"
    assert aws.config.secret_key, "Appease typechecker"

    e2e_test_job_vars: NomadVars = {
        "analyzer_bucket": grapl_stack.analyzer_bucket,
        "aws_access_key_id": aws.config.access_key,
        "aws_access_key_secret": aws.config.secret_key,
        "_aws_endpoint": grapl_stack.aws_endpoint,
        "aws_region": aws.get_region().name,
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

    integration_test_job_vars: NomadVars = {
        "aws_access_key_id": aws.config.access_key,
        "aws_access_key_secret": aws.config.secret_key,
        "_aws_endpoint": grapl_stack.aws_endpoint,
        "aws_region": aws.get_region().name,
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


if __name__ == "__main__":
    main()
