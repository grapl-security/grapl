import os
import re
from pathlib import Path
from typing import Mapping, Optional, Sequence, Type, TypeVar, cast

import pulumi_aws as aws
from typing_extensions import Final

import pulumi

T = TypeVar("T", bound=object)

# This will be incorporated into various infrastructure object names.
DEPLOYMENT_NAME = pulumi.get_stack()

# This must be the same as the value defined in local-grapl.env
GRAPL_TEST_USER_NAME = f"{DEPLOYMENT_NAME}-grapl-test-user"

# Sometimes we need to refer to other code or artifacts relative to
# the repository root.
REPOSITORY_ROOT = os.path.join(os.path.dirname(__file__), "../..")


def repository_path(relative_path: str) -> Path:
    """
    Resolve `relative_path` relative to the root of the repository.
    """
    return Path(os.path.join(REPOSITORY_ROOT), relative_path).resolve()


def _validate_deployment_name() -> None:
    # ^ and $ capture the whole string: start and end
    # Must start with an alpha
    # Must end with an alpha or number
    # In the middle, - and _ are fine
    regex = re.compile("^[a-z]([a-z0-9_-]?[a-z0-9]+)*$")
    if regex.match(DEPLOYMENT_NAME) is None:
        raise ValueError(
            f"Deployment name {DEPLOYMENT_NAME} is invalid - should match regex {regex}."
        )


_validate_deployment_name()

# Use this to modify behavior or configuration for provisioning in
# Local Grapl (as opposed to any other real deployment)
LOCAL_GRAPL = DEPLOYMENT_NAME == "local-grapl"

# For importing some objects, we have to construct a URL, ARN, etc
# that includes the AWS account ID.
AWS_ACCOUNT_ID = "000000000000" if LOCAL_GRAPL else aws.get_caller_identity().account_id

GLOBAL_LAMBDA_ZIP_TAG = os.getenv("TAG", "latest")
"""Filename tag for all lambda function ZIP files.

All our lambda function ZIP files currently have a name like:

    <LAMBDA_NAME>-<TAG>.zip

Since all the lambdas share the same tag, we can specify this globally.

Use the environment variable `TAG` to specify a tag, or
leave it unset to use the default value of `latest`.

"""


SERVICE_LOG_RETENTION_DAYS: Final[int] = 30

DGRAPH_LOG_RETENTION_DAYS: Final[int] = 7

DEFAULT_ENVVARS = {
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
            f"Add it to env variables or `Pulumi.{DEPLOYMENT_NAME}.yaml`."
        )
    return value


# Boy, this env name was not forward-thinking
OPERATIONAL_ALARMS_EMAIL = _require_env_var("GRAPL_CDK_OPERATIONAL_ALARMS_EMAIL")


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

        Please edit your Pulumi.{DEPLOYMENT_NAME}.yaml file and add the following:

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
    return version


def require_artifact(artifact_name: str, klass: Type[T]) -> T:
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
    if isinstance(artifact, klass):
        return artifact
    else:
        raise ArtifactException(
            f"Expected artifact to be a {klass} but was {type(artifact)}"
        )
