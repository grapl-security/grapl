import os
from typing import NewType, Optional

from infra.artifacts import ArtifactGetter

DockerImageId = NewType("DockerImageId", str)
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


def _version_tag(
    key: str,
    artifacts: ArtifactGetter,
) -> str:
    """
    First, try and get the value from artifacts;
        if no artifact and require_artifact, throw error
    then fall back to $TAG, and fail if it isn't set.

    We generally set `require_artifact=True` for production deployments.
    """
    artifact_version = artifacts.get(key)
    if artifact_version:
        return artifact_version

    tag = os.environ["TAG"]
    assert (
        tag != "latest"
    ), "Never try to deploy from a 'latest' tag! Plus, Nomad can't access these from the local host, making local development problematic"
    return tag


class DockerImageIdBuilder:
    def __init__(
        self, container_repository: Optional[str], artifacts: ArtifactGetter
    ) -> None:
        self.container_repository = (
            f"{container_repository}/" if container_repository else ""
        )
        self.artifacts = artifacts

    def build(self, image_name: str, tag: str) -> DockerImageId:
        return DockerImageId(f"{self.container_repository}{image_name}:{tag}")

    def build_with_tag(self, image_name: str) -> DockerImageId:
        """
        Automatically grabs the version tag from config's artifacts.
        """
        tag = _version_tag(image_name, artifacts=self.artifacts)
        return self.build(image_name, tag)
