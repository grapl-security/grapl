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


def _docker_version_tag_from_env() -> str:
    """
    If a tag isn't specified in `artifacts:`, fall back to os.environ["TAG"].
    Only applicable to local-grapl.
    """
    tag = os.environ["IMAGE_TAG"]
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

    def build(
        self, container_repository: str, image_name: str, tag: str
    ) -> DockerImageId:
        return DockerImageId(f"{container_repository}{image_name}:{tag}")

    def build_with_tag(self, image_name: str) -> DockerImageId:
        """
        Automatically grabs the version tag from config's artifacts.
        """
        artifact_version = self.artifacts.get(image_name)
        if artifact_version:
            return self.build(
                container_repository=self.container_repository,
                image_name=image_name,
                tag=artifact_version,
            )
        else:
            # This is only possible on Local Grapl, in which case we assume
            # we're using a local image - even if the container repository
            # is specified.
            tag = _docker_version_tag_from_env()
            return self.build(
                container_repository="",  # local Docker registry
                image_name=image_name,
                tag=tag,
            )
