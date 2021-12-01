import os
from typing import Mapping, NewType, Optional

from typing_extensions import Final

# This default is chosen because Nomad cannot pull images called "latest"
# from the local machine (it takes it as a directive to go to Dockerhub)
# Originates at the `TAG ?= dev` at the top of the Makefile.
_DEFAULT_TAG: Final[str] = "dev"


"""
A Docker image identifier is something that can be consumed by the
Nomad Docker plugin `image` field.
https://www.nomadproject.io/docs/drivers/docker#image
The values can look like, for instance:
- a hardcoded value pulled from Dockerhub
    "dgraph/dgraph:v21.0.3"
- an image pulled from the host's Docker daemon (no `:latest`!)
    "model-plugin-deployer:dev"
- an image pulled from Cloudsmith
    "docker.cloudsmith.io/grapl/raw/graph-merger:20211105192234-a86a8ad2"
"""
DockerImageId = NewType("DockerImageId", str)


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


class DockerImageIdBuilder:
    def __init__(self, container_repository: Optional[str]) -> None:
        self.container_repository = (
            f"{container_repository}/" if container_repository else ""
        )

    def build(self, image_name: str, tag: str) -> DockerImageId:
        return DockerImageId(f"{self.container_repository}{image_name}:{tag}")
