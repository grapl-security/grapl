import json
from pathlib import Path
from typing import Any, Awaitable, Mapping, Optional

import pulumi_nomad as nomad
from infra.config import DEPLOYMENT_NAME

import pulumi


class NomadJob(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        jobspec: Path,
        vars: Mapping[str, Any],
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:NomadJob", name, None, opts)

        self.job = nomad.Job(
            resource_name=f"{DEPLOYMENT_NAME}-{name}-job",
            jobspec=self._file_contents(str(jobspec)),
            hcl2=nomad.JobHcl2Args(enabled=True, vars=self._fix_pulumi_preview(vars)),
            opts=pulumi.ResourceOptions(parent=self),
            # Wait for all services to become healthy
            detach=False,
        )

    def _file_contents(self, nomad_file: str) -> str:
        with open(nomad_file, "r") as f:
            jobspec = f.read()
            f.close()
            return jobspec

    def _fix_pulumi_preview(self, vars: Mapping[str, Any]) -> Mapping[str, Any]:
        """
        This is an ugly hack to deal with pulumi preview never resolving Outputs into a real string.
        Without this, the vars gets unset if there's a single key with an unresolved output
        """
        if pulumi.runtime.is_dry_run():
            pulumi_preview_replacement_string = "PULUMI_PREVIEW_STRING"
            _redis_endpoint = "redis://LOCAL_GRAPL_REPLACE_IP:6379"

            nomad_vars = {}
            for key, value in vars.items():
                pulumi.log.info(key)
                if isinstance(value, pulumi.Output):
                    # TODO figure out a better way to filter down to output<string>

                    value = pulumi_preview_replacement_string
                    # special rule since we split the redis endpoint
                    if key == "_redis_endpoint":
                        value = _redis_endpoint

                nomad_vars[key] = value
            return nomad_vars
        else:
            return vars
