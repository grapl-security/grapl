import json
from pathlib import Path
from typing import Mapping, Optional, Union

import pulumi_nomad as nomad
from infra.config import DEPLOYMENT_NAME

import pulumi

_ValidNomadVarTypes = pulumi.Input[Union[str, bool, int, Mapping[str, str]]]
NomadVars = Mapping[str, Optional[_ValidNomadVarTypes]]


class NomadJob(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        jobspec: Path,
        vars: NomadVars,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:NomadJob", name, None, opts)

        vars = self._fix_pulumi_preview(vars)
        vars = self._json_dump_complex_types(vars)

        self.job = nomad.Job(
            resource_name=f"{DEPLOYMENT_NAME}-{name}-job",
            jobspec=self._file_contents(str(jobspec)),
            hcl2=nomad.JobHcl2Args(enabled=True, vars=self._fix_pulumi_preview(vars)),
            # Wait for all services to become healthy
            detach=False,
            # Purge job from Nomad servers after a `pulumi destroy`
            purge_on_destroy=True,
            opts=pulumi.ResourceOptions(
                parent=self, custom_timeouts=opts.custom_timeouts if opts else None
            ),
        )

        self.register_outputs({})

    def _file_contents(self, nomad_file: str) -> str:
        with open(nomad_file, "r") as f:
            jobspec = f.read()
            f.close()
            return jobspec

    def _json_dump_complex_types(self, vars: NomadVars) -> NomadVars:
        """
        Nomad accepts input of lists and maps, but the Nomad/Pulumi plugin doesn't
        convert them correctly.
        """
        return {
            k: json.dumps(v) if isinstance(v, (dict, list)) else v
            for (k, v) in vars.items()
        }

    def _fix_pulumi_preview(self, vars: NomadVars) -> NomadVars:
        """
        This is an ugly hack to deal with pulumi preview never resolving Outputs into a real string.
        Without this, the vars gets unset if there's a single key with an unresolved output
        """
        if pulumi.runtime.is_dry_run():
            pulumi_preview_replacement_string = "PULUMI_PREVIEW_STRING"
            _redis_endpoint = "redis://LOCAL_GRAPL_REPLACE_IP:6379"

            nomad_vars = {}
            for key, value in vars.items():
                if isinstance(value, pulumi.Output):
                    # TODO figure out a better way to filter down to output<string> and not just all outputs

                    value = pulumi_preview_replacement_string
                    # special rule since we split the redis endpoint
                    if key == "_redis_endpoint":
                        value = _redis_endpoint

                nomad_vars[key] = value
            return nomad_vars
        else:
            return vars
