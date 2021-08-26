from typing import Dict, Optional
from infra.config import DEPLOYMENT_NAME
import pulumi_nomad as nomad
import pulumi


class NomadJob(pulumi.ComponentResource):
    def __init__(self, name: str, vars: Dict[str, str], opts: Optional[pulumi.ResourceOptions] = None) -> None:
        super().__init__("grapl:NomadJob", name, None, opts)

        nomad_file = "/home/grapl/nomad/grapl-core.nomad"
        with open(nomad_file) as f:
            jobspec = f.read()

        self.grapl_core = nomad.Job(
            resource_name=f"{DEPLOYMENT_NAME}-grapl-core-job",
            jobspec=jobspec,
            hcl2=nomad.JobHcl2Args(enabled=True, vars=vars),
            opts=pulumi.ResourceOptions(parent=self)
        )

