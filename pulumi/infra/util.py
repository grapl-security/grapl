from typing import Any

import pulumi

# Use this to modify behavior or configuration for provisioning in
# Local Grapl (as opposed to any other real deployment)
IS_LOCAL = pulumi.get_stack() == "local-grapl"


# Yes I hate the 'Any' type just as much as you do, but there's
# apparently not a way to type kwargs right now.
def import_aware_opts(resource_id: str, **kwargs: Any) -> pulumi.ResourceOptions:
    """Pass the resource ID that corresponds to a particular resource
    when you're importing from existing infrastructure, as well as any
    other kwargs that `pulumi.ResourceOptions` would accept.

    If the Pulumi stack is currently configured to import (rather than
    create), the appropriate configuration will be injected into the
    `ResourceOptions` that is returned.

    Otherwise, a `ResourceOptions` constructed from the given kwargs
    will be returned.

    This should be used to generate `ResourceOptions` for *all* resources!

    To enable importing, rather than creating, set the following
    config for the stack:

        pulumi config set grapl:import_from_existing True

    """

    import_from_existing = pulumi.Config().require_bool("import_from_existing")

    given_opts = pulumi.ResourceOptions(**kwargs)
    import_opts = pulumi.ResourceOptions(
        import_=resource_id if import_from_existing else None
    )
    return pulumi.ResourceOptions.merge(given_opts, import_opts)
