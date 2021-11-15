from pathlib import Path
from typing import Optional

import pulumi_consul as consul

import pulumi


class ConsulAclPolicies(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        acl_directory: Path,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:ConsulAclPolicies", "name", None, opts)

        # Autogenerate policies from files
        self.policies = {}
        files = list(Path(acl_directory).glob("*.hcl"))
        for file in files:
            hcl_txt = Path(file).read_text()
            self.policies[file.stem] = consul.AclPolicy(
                f"{name}-{file.stem}",
                name=f"{name}-{file.stem}",
                rules=hcl_txt,
                opts=pulumi.ResourceOptions(parent=self),
            )

        self.register_outputs({})
