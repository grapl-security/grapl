from typing import Any, Mapping, cast

import pulumi_consul as consul
import pulumi_nomad as nomad

import pulumi


# We're using the Any typehint here since mypy doesn't YET support modules being subtypes of protocols per
# https://github.com/python/mypy/issues/5018 :(
def get_hashicorp_provider_address(
    pulumi_class: Any,
    provider_type: str,
    stack: pulumi.StackReference,
    additional_configs: Mapping[str, Any] = {},
) -> Any:
    """
    This supports getting a Provider object with an explicit address set.
    This will take the address from the pulumi config file if it is set or fall back to the address in the stack refererence. This allows using SSM port tunneling when run from a workstation.
    :param pulumi_class: Should be one of pulumi_consul or pulumi_nomad
    :param provider_type: One of "consul", "nomad"
    :param stack: The corresponding stack reference
    :return: pulumi.providerReference. Should be used in opts
    """
    override_address = pulumi.Config(provider_type).get("address")
    address = override_address or stack.require_output("address")
    return pulumi_class.Provider(provider_type, address=address, **additional_configs)


def get_nomad_provider_address(
    stack: pulumi.StackReference,
    additional_configs: Mapping[str, Any] = {},
) -> nomad.Provider:
    return cast(
        nomad.Provider,
        get_hashicorp_provider_address(
            pulumi_class=nomad,
            provider_type="nomad",
            stack=stack,
            additional_configs=additional_configs,
        ),
    )


def get_consul_provider_address(
    stack: pulumi.StackReference,
    additional_configs: Mapping[str, Any] = {},
) -> consul.Provider:
    return cast(
        consul.Provider,
        get_hashicorp_provider_address(
            pulumi_class=consul,
            provider_type="consul",
            stack=stack,
            additional_configs=additional_configs,
        ),
    )
