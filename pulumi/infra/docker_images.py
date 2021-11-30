import os
from typing import Mapping, Optional

from typing_extensions import Final

# This default is chosen because Nomad cannot pull images called "latest"
# from the local machine (it takes it as a directive to go to Dockerhub)
# Originates at the `TAG ?= dev` at the top of the Makefile.
_DEFAULT_TAG: Final[str] = "dev"


def version_tag(
    key: str,
    artifacts: Mapping[str, str],
    require_artifact: bool = False,
) -> str:
    """
    First, try and get the value from artifacts;
        if no artifact and require_artifact, throw error
    then fall back to $TAG;
    then fall back to "dev"

    We generally set `require_artifact=True` for production deployments.
    """
    artifact_version = artifacts.get(key)
    if artifact_version:
        return artifact_version
    if not artifact_version and require_artifact:
        raise KeyError(
            "Expected to find an artifacts entry for {key} in Pulumi config file"
        )

    tag = os.getenv("TAG")
    if tag:
        return tag

    return _DEFAULT_TAG


class CloudsmithImageUrl:
    def __init__(self, container_repository: Optional[str]) -> None:
        if container_repository is not None:
            self.container_repository: str = f"{container_repository}/"
        else:
            self.container_repository = ""

    def build(self, image_name: str, tag: str) -> str:
        return f"{self.container_repository}{image_name}:{tag}"
