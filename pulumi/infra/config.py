import os
import re
from pathlib import Path
from typing import Mapping

import pulumi_aws as aws
from typing_extensions import Final

import pulumi

# This will be incorporated into various infrastructure object names.
STACK_NAME: Final[str] = pulumi.get_stack()

# Only use this value for helpful error messages;
# Pulumi is responsible for actually accessing and parsing this file.
STACK_CONFIG_FILENAME: Final[str] = f"Pulumi.{STACK_NAME}.yaml"

# This must be the same as the value defined in local-grapl.env
GRAPL_TEST_USER_NAME: Final[str] = f"{STACK_NAME}-grapl-test-user"

# note: this ${} is interpolated inside Nomad
HOST_IP_IN_NOMAD: Final[str] = "${attr.unique.network.ip-address}"

# This is equivalent to what "${attr.unique.network.ip-address}" resolves to but is used for cases where variable
# interpolation is not available. Currently this is used to get the private IP for Nomad so the otel collector can
# scrape metrics
LOCAL_HOST_IP: Final[str] = os.getenv("LOCAL_HOST_ETH0_IP")


def repository_path(relative_path: str) -> Path:
    """
    Resolve `relative_path` relative to the root of the repository.
    """
    repository_root = os.path.join(os.path.dirname(__file__), "../..")
    return Path(os.path.join(repository_root), relative_path).resolve()


def _validate_stack_name() -> None:
    # ^ and $ capture the whole string: start and end
    # Must start with an alpha
    # Must end with an alpha or number
    # In the middle, - and _ are fine
    regex = re.compile("^[a-z]([a-z0-9_-]?[a-z0-9]+)*$")
    if regex.match(STACK_NAME) is None:
        raise ValueError(
            f"Deployment name {STACK_NAME} is invalid - should match regex {regex}."
        )


_validate_stack_name()

# Use this to modify behavior or configuration for provisioning in
# Local Grapl (as opposed to any other real deployment)

LOCAL_GRAPL: Final[bool] = STACK_NAME in (
    "local-grapl",
    "local-grapl-python-integration-tests",
    "local-grapl-rust-integration-tests",
)
# (We have a different one for integration tests because `pulumi login --local`
#  doesn't allow for stack name conflicts, even across projects.)

# A "real" deployment is one that will be deployed in our CI/CD
# environment, not a developer sandbox environment.
#
# (At the moment, we only have "testing"; this can grow to include
# other deployments in the future. Another option would be to declare
# a convention for developer sandbox environments and have logic pivot
# on that, instead.)
REAL_DEPLOYMENT: Final[bool] = STACK_NAME in ("testing")

# For importing some objects, we have to construct a URL, ARN, etc
# that includes the AWS account ID.
AWS_ACCOUNT_ID: Final[str] = (
    "000000000000" if LOCAL_GRAPL else aws.get_caller_identity().account_id
)


DEFAULT_ENVVARS: Final[Mapping[str, str]] = {
    "GRAPL_LOG_LEVEL": "DEBUG",
    "RUST_LOG": "DEBUG",
    "RUST_BACKTRACE": "0",
}


def _require_env_var(key: str) -> str:
    """
    Grab a key from env variables, or fallback to Pulumi.xyz.yaml
    """
    value = os.getenv(key) or pulumi.Config().get(key)
    if not value:
        raise KeyError(
            f"Missing environment variable '{key}'. "
            f"Add it to env variables or `{STACK_CONFIG_FILENAME}`."
        )
    return value


def get_grapl_ops_alarms_email() -> str:
    return _require_env_var("GRAPL_OPERATIONAL_ALARMS_EMAIL")


def configurable_envvar(service_name: str, var: str) -> str:
    """Look up the desired environment variable in Pulumi configuration for the given service or return a default value.

    Your configuration YAML should look like this:

    config:
      grapl:env_vars:
        <SERVICE_NAME>:
          <VARIABLE_NAME>: <VARIABLE_VALUE>
    """
    config_key = "env_vars"
    vars = (pulumi.Config().get_object(config_key) or {}).get(service_name, {})
    value = vars.get(var) or DEFAULT_ENVVARS.get(var)
    if not value:
        raise ValueError(
            f"""
        You have tried to retrieve a value for the '{var}' environment variable for the
        '{service_name}' service, but we have no record of this variable!

        Please edit your `{STACK_CONFIG_FILENAME}` file and add the following:

        config:
          {pulumi.get_project()}:{config_key}:
            {service_name}:
              {var}: <YOUR_DESIRED_VALUE>

        If '{var}' is a common variable shared across many services, consider adding it to
        the DEFAULT_ENVVARS dictionary in {__file__} instead.

        """
        )
    else:
        return value


def cloudsmith_repository_name() -> str | None:
    """The repository from which to pull container images and Firecracker
    packages from.

    This will be different for different stacks; we promote packages
    through a series of different registries that mirrors the progress
    of code through our pipelines.

    The value will be something like `grapl/testing`.
    """
    return pulumi.Config().get("cloudsmith-repository-name")


def container_repository() -> str | None:
    """The repository from which to pull container images from.

    Not specifying a repository will result in local images being used,
    but only for local-grapl stacks.
    """

    repo_name = cloudsmith_repository_name()
    if repo_name:
        return f"docker.cloudsmith.io/{repo_name}"
    return None
