import os
import re
from pathlib import Path
from typing import Any, Mapping, Optional, Sequence, Union

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

# Sometimes we need to refer to other code or artifacts relative to
# the repository root.
REPOSITORY_ROOT: Final[str] = os.path.join(os.path.dirname(__file__), "../..")

# note: this ${} is interpolated inside Nomad
HOST_IP_IN_NOMAD: Final[str] = "${attr.unique.network.ip-address}"

# This is equivalent to what "${attr.unique.network.ip-address}" resolves to but is used for cases where variable
# interpolation is not available such as network.dns prior to Nomad 1.3.0
# Hax: If this is not available such as in Buildkite, we'll default to Google DNS for now.
# This should be going away once https://github.com/hashicorp/nomad/pull/12817 is merged
CONSUL_DNS_IP: Final[str] = os.getenv("CONSUL_DNS_IP", "8.8.8.8")


def to_bool(input: Optional[Union[str, bool]]) -> Optional[bool]:
    if isinstance(input, bool):
        return input
    elif input is None:
        return None
    elif input in ("True", "true"):
        return True
    elif input in ("False", "false"):
        return False
    else:
        raise ValueError(f"Invalid bool value: {repr(input)}")


def repository_path(relative_path: str) -> Path:
    """
    Resolve `relative_path` relative to the root of the repository.
    """
    return Path(os.path.join(REPOSITORY_ROOT), relative_path).resolve()


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
    "local-grapl-integration-tests",
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

SERVICE_LOG_RETENTION_DAYS: Final[int] = 30

DGRAPH_LOG_RETENTION_DAYS: Final[int] = 7

DEFAULT_ENVVARS: Final[Mapping[str, str]] = {
    "GRAPL_LOG_LEVEL": "DEBUG",
    "RUST_LOG": "DEBUG",
    "RUST_BACKTRACE": "0",
}


class ArtifactException(Exception):
    pass


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


def configurable_envvars(service_name: str, vars: Sequence[str]) -> Mapping[str, str]:
    """Return a map of environment variable values for the named service,
    pulled from Pulumi configuration or default values, suitable for
    splicing into other environment maps for configuring services.
    """
    return {v: configurable_envvar(service_name, v) for v in vars}


# TODO: The verbiage "version" here is a bit restrictive.
def configured_version_for(artifact_name: str) -> Optional[str]:
    """Given the name of an artifact, retrieves the version of that
    artifact from the current stack configuration. Returns `None` if
    no version has been specified for that artifact.

    In general, we will have versions specified for all artifacts when
    doing deploys of release candidates to automated testing and
    production infrastructure. However, individual developers working
    on features _may_ wish to specify concrete versions for some
    artifacts, while leaving others unspecified. In the latter case,
    artifacts built locally from the current code checkout will be
    used instead. This allows developers to deploy the code they are
    currently iterating on to their own sandbox environments.

    """
    artifacts = pulumi.Config().get_object("artifacts") or {}
    version = artifacts.get(artifact_name)

    if (not version) and REAL_DEPLOYMENT:
        raise Exception(
            f"""
        Tried to deploy the {STACK_NAME} stack, but no version for {artifact_name} was found!

        This stack must have a version configured for ALL artifacts that it uses.
        """
        )

    return version


# This should be (x: str, y: Type[T]) -> T, but: https://github.com/python/mypy/issues/9773
def require_artifact(artifact_name: str) -> Any:
    """
    Given the name of an artifact, retrieves the value of that
    artifact from the current stack configuration.
    Raise a helpful exception if no entry is found for that artifact.
    """
    artifacts = pulumi.Config().get_object("artifacts") or {}
    artifact = artifacts.get(artifact_name)
    if artifact is None:
        raise ArtifactException(
            f"We couldn't find an artifact named {artifact_name} in your stack."
            "\nYou likely want to run `pulumi/bin/copy_artifacts_from_rc.sh`, which"
            " will grab concrete artifact values from our latest `origin/rc` branch."
            "\nDon't forget to remove artifacts you don't need after running it!"
        )
    return artifact


def cloudsmith_repository_name() -> Optional[str]:
    """The repository from which to pull container images and Firecracker
    packages from.

    This will be different for different stacks; we promote packages
    through a series of different registries that mirrors the progress
    of code through our pipelines.

    The value will be something like `grapl/testing`.
    """
    return pulumi.Config().get("cloudsmith-repository-name")


def container_repository() -> Optional[str]:
    """The repository from which to pull container images from.

    Not specifying a repository will result in local images being used,
    but only for local-grapl stacks.
    """

    repo_name = cloudsmith_repository_name()
    if repo_name:
        return f"docker.cloudsmith.io/{repo_name}"
    return None
