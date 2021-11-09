import json
import os
import sys
from pathlib import Path

sys.path.insert(0, "..")

import pulumi_nomad as nomad
from infra import config
from infra.autotag import register_auto_tags
from infra.nomad_job import NomadJob, NomadVars
from infra.quiet_docker_build_output import quiet_docker_output

import pulumi
import pulumi_aws as aws


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

        assert aws.config.access_key, "Appease typechecker"
        assert aws.config.secret_key, "Appease typechecker"

        self.e2e_test_job_vars: NomadVars = {
            "analyzer_bucket": output("analyzers-bucket"),
            "aws_access_key_id": aws.config.access_key,
            "aws_access_key_secret": aws.config.secret_key,
            "_aws_endpoint": output("aws-endpoint"),
            "aws_region": aws.get_region().name,
            "deployment_name": output("deployment-name"),
            "schema_properties_table_name": output("schema-properties-table"),
            "sysmon_log_bucket": output("sysmon-log-bucket"),
            "schema_table_name": output("schema-table"),
            "sysmon_generator_queue": output("sysmon-generator-queue"),
            "test_user_name": output("test-user-name"),
        }

        self.integration_test_job_vars: NomadVars = {
            "_aws_endpoint": output("aws-endpoint"),
            "_kafka_endpoint": output("kafka-endpoint"),
            "_redis_endpoint": output("redis-endpoint"),
            "aws_access_key_id": aws.config.access_key,
            "aws_access_key_secret": aws.config.secret_key,
            "aws_region": aws.get_region().name,
            "deployment_name": output("deployment-name"),
            "schema_properties_table_name": output("schema-properties-table"),
            "test_user_name": output("test-user-name"),
            "grapl_root": os.environ["GRAPL_ROOT"],
            "docker_user": os.environ["DOCKER_USER"],
        }



def main() -> None:
    ##### Preamble

    stack_name = stackname_sans_prefix()

    quiet_docker_output()

    # These tags will be added to all provisioned infrastructure
    # objects.
    register_auto_tags({"grapl deployment": config.DEPLOYMENT_NAME})

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

    e2e_tests = NomadJob(
        "e2e-tests",
        jobspec=Path("../../nomad/local/e2e-tests.nomad").resolve(),
        vars=grapl_stack.e2e_test_job_vars,
    )

    integration_tests = NomadJob(
        "integration-tests",
        jobspec=Path("../../nomad/local/integration-tests.nomad").resolve(),
        vars=grapl_stack.integration_test_job_vars,
    )


if __name__ == "__main__":
    main()
