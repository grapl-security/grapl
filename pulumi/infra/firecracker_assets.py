from pathlib import Path

from infra import config
from infra.artifacts import ArtifactGetter
from infra.path import path_from_root

import pulumi


class FirecrackerAssets(pulumi.ComponentResource):
    """
    A Postgres instance running in RDS.
    """

    def __init__(
        self,
        name: str,
        repository: str,
        artifacts: ArtifactGetter,
        opts: pulumi.ResourceOptions,
    ) -> None:
        super().__init__("grapl:FirecrackerAssets", name, None, opts)

        child_opts = pulumi.ResourceOptions(parent=self)

        self.firecracker_kernel_asset = local_or_remote_asset(
            local_path=path_from_root("dist/firecracker_kernel.tar.gz"),
            artifacts=artifacts,
            artifact_key="firecraker_kernel.tar.gz",
            repository=repository,
            opts=child_opts,
        )


def cloudsmith_cdn_uri(
    repository: str,
    package_name: str,
    version: str,
) -> str:
    return (
        f"https://dl.cloudsmith.io/public/{repository}"
        f"/raw/versions/{version}/{package_name}"
    )


def local_or_remote_asset(
    local_path: Path,
    artifacts: ArtifactGetter,
    artifact_key: str,
    repository: str,
    opts: pulumi.ResourceOptions,
) -> pulumi.asset.Asset:
    # First, allow a local asset if it's local-grapl
    if config.LOCAL_GRAPL:
        if local_path.resolve().exists():
            return pulumi.asset.FileAsset(local_path)

    # Or fall back to a remote path
    version = artifacts.get(artifact_key)
    if version:
        uri = cloudsmith_cdn_uri(
            repository=repository,
            package_name=artifact_key,
            version=version,
        )
        return pulumi.asset.RemoteAsset(uri)

    raise ValueError(
        f"Couldn't find an asset at {local_path}"
        f" or an artifact with key {artifact_key}"
    )
