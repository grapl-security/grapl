import sys
from pathlib import Path

sys.path.insert(0, "..")

import pulumi_nomad as nomad
from infra import config

from infra.autotag import register_auto_tags

from infra.nomad_job import NomadJob, NomadVars
from infra.quiet_docker_build_output import quiet_docker_output

import pulumi

def stackname_sans_prefix() -> str:
    real_stackname = pulumi.get_stack()
    prefix = "grapl/networking"
    split = real_stackname.split(prefix)
    assert len(split) == 2, f"Expected a stack prefix of {prefix}, found {real_stackname}"
    return split[1]

def main() -> None:
    ##### Preamble

    stack_name = stackname_sans_prefix(pulumi.get_stack())

    quiet_docker_output()

    # These tags will be added to all provisioned infrastructure
    # objects.
    register_auto_tags({"grapl deployment": config.DEPLOYMENT_NAME})

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

    grapl_stack = pulumi.StackReference(
        f"grapl/{stack_name}",
    )

    e2e_test_job_vars: pulumi.Output[NomadVars] = grapl_stack.require_output("e2e-test-job-vars")

    e2e_tests = NomadJob(
        "e2e-tests",
        jobspec=Path("../../nomad/local/e2e-tests.nomad").resolve(),
        vars=e2e_test_job_vars,
    )

if __name__ == "__main__":
    main()
