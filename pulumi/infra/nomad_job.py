from typing import Optional

import pulumi_nomad as nomad
from infra.config import DEPLOYMENT_NAME

import pulumi


class NomadJob(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        vars: pulumi.Output,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:NomadJob", name, None, opts)

        self.grapl_core = nomad.Job(
            resource_name=f"{DEPLOYMENT_NAME}-grapl-core-job",
            jobspec=self._file("../../nomad/grapl-core.nomad"),
            hcl2=nomad.JobHcl2Args(enabled=True, vars=vars),
            opts=pulumi.ResourceOptions(parent=self),
        )

    def _file(self, nomad_file: str) -> str:
        with open(nomad_file) as f:
            jobspec = f.read()
            f.close()
            return jobspec
