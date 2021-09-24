from pathlib import Path
from typing import Optional

import pulumi_nomad as nomad
from infra.config import DEPLOYMENT_NAME

import pulumi


class NomadJob(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        jobspec: Path,
        vars: pulumi.Output,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:NomadJob", name, None, opts)

        self.job = nomad.Job(
            resource_name=f"{DEPLOYMENT_NAME}-{name}-job",
            jobspec=self._file_contents(str(jobspec)),
            hcl2=nomad.JobHcl2Args(enabled=True, vars=vars),
            opts=pulumi.ResourceOptions(parent=self),
            # Wait for all services to become healthy
            detach=False,
        )

    def _file_contents(self, nomad_file: str) -> str:
        with open(nomad_file, "r") as f:
            jobspec = f.read()
            f.close()
            return jobspec

    @staticmethod
    def image_name(container_name: str,registry: str = "", org: str = "", repo: str = "", tag: str) -> str:
        return f"{registry}{org}/{repo}{container_name}:{tag}"

