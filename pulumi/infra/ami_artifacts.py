from typing import Dict, Union

import pulumi_aws as aws
from infra.config import ArtifactException, require_artifact
from typing_extensions import Literal

PackerImageName = Union[
    # Compare with image.pkr.hcl's variable "image_name"
    Literal["grapl-nomad-consul-server"],
    Literal["grapl-nomad-consul-client"],
]


def get_ami_id(packer_image_name: PackerImageName) -> str:
    """
    Grab AMI IDs from your stack file (Pulumi.stackname.yaml).
    There's a good chance those values don't exist for you yet, in which case,
    follow the instructions in `require_artifact`.
    """
    region_to_ami: Dict[str, str] = require_artifact(f"{packer_image_name}")
    region = aws.get_region().name
    ami = region_to_ami.get(region, None)
    if ami is None:
        raise ArtifactException(f"Couldn't find an AMI for region {region}")
    return ami
