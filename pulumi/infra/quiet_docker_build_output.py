"""
Once `docker buildx build` truly supports a `--quiet`, we can remove all this.
https://github.com/docker/buildx/issues/621
"""
from __future__ import annotations

import inspect
import os
from typing import List, Optional
from unittest.mock import patch

import pulumi


def quiet_docker_output() -> None:
    """
    Replace `pulumi.log.warn` with a function that quiets any `msg`
    called from the module `pulumi_docker`.
    Disable behavior with VERBOSE_DOCKER_OUTPUT=1
    """
    skip_quiet = os.getenv("VERBOSE_DOCKER_OUTPUT", default=0)
    if skip_quiet:
        return

    original_warn = pulumi.log.warn

    def replacement_warn(
        msg: str,
        resource: Optional[pulumi.Resource] = None,
        stream_id: Optional[int] = None,
        ephemeral: Optional[bool] = None,
    ) -> None:
        # Same method signature as original_warn
        callers = first_n_callers_in_stack(n=6)
        if any("pulumi_docker" in module_name for module_name in callers):
            msg = "(redacted Docker spew from `quiet_docker_output`)"
        original_warn(
            msg=msg, resource=resource, stream_id=stream_id, ephemeral=ephemeral
        )

    patcher = patch.object(pulumi.log, pulumi.log.warn.__name__).start()
    patcher.side_effect = replacement_warn


def first_n_callers_in_stack(n: int) -> List[str]:
    """
    Get the names of the modules in the current stack call context
    """
    caller_stacks = [inspect.stack()[i] for i in range(1, n)]
    caller_modules = [inspect.getmodule(stack[0]) for stack in caller_stacks]
    caller_module_names = [
        module.__name__ for module in caller_modules if module  # filter nones
    ]
    return caller_module_names
