import os
import re
from typing import Any, Mapping, Sequence, Tuple

import pulumi_aws as aws
from typing_extensions import Final

import pulumi

# This will be incorporated into various infrastructure object names.
DEPLOYMENT_NAME = pulumi.get_stack()


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

GLOBAL_LAMBDA_ZIP_TAG = os.getenv("GRAPL_LAMBDA_TAG", "latest")
"""Filename tag for all lambda function ZIP files.

All our lambda function ZIP files currently have a name like:

    <LAMBDA_NAME>-<TAG>.zip

Since all the lambdas share the same tag, we can specify this globally.

Use the environment variable `GRAPL_LAMBDA_TAG` to specify a tag, or
leave it unset to use the default value of `latest`.

"""


SERVICE_LOG_RETENTION_DAYS: Final[int] = 30

DGRAPH_LOG_RETENTION_DAYS: Final[int] = 7

DEFAULT_ENVVARS = {
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


def model_plugin_deployer_host_port() -> Tuple[str, int]:
    """Return the host, port pair for the model-plugin-deployer. This
    is a hack that should be removed once the model-plugin-deployer
    lambda is migrated to Pulumi.

    """
    config = pulumi.Config()
    host = config.get("GRAPL_MODEL_PLUGIN_DEPLOYER_HOST")
    if host is None:
        raise Exception("GRAPL_MODEL_PLUGIN_DEPLOYER_HOST is unset")
    port = config.get_int("GRAPL_MODEL_PLUGIN_DEPLOYER_PORT")
    if port is None:
        raise Exception("GRAPL_MODEL_PLUGIN_DEPLOYER_PORT is unset")
    return host, port


def grapl_api_host_port() -> Tuple[str, int]:
    """Return the host, port pair for the model-plugin-deployer. This
    is a hack that should be removed once the Grapl API is migrated to Pulumi.

    """
    config = pulumi.Config()
    host = config.get("GRAPL_API_HOST")
    if host is None:
        raise Exception("GRAPL_API_HOST is unset")
    port = config.get_int("GRAPL_HTTP_FRONTEND_PORT")
    if port is None:
        raise Exception("GRAPL_HTTP_FRONTEND_PORT is unset")
    return host, port


def grapl_graphql_host_port() -> Tuple[str, int]:
    """Return the host, port pair for the model-plugin-deployer. This
    is a hack that should be removed once the graphql service is migrated to Pulumi.

    """
    config = pulumi.Config()
    host = config.get("GRAPL_GRAPHQL_HOST")
    if host is None:
        raise Exception("GRAPL_GRAPHQL_HOST is unset")
    port = config.get_int("GRAPL_GRAPHQL_PORT")
    if port is None:
        raise Exception("GRAPL_GRAPHQL_PORT is unset")
    return host, port
