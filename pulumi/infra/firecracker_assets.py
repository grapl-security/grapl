from pathlib import Path
from typing import Optional

from infra import config
from infra.artifacts import ArtifactGetter
from infra.path import path_from_root

import pulumi


class FirecrackerAssets(pulumi.ComponentResource):
    """
    Uploads Firecracker assets from disk-or-Cloudsmith into
    S3 bucket.
    """

    def __init__(
        self,
        name: str,
        repository_name: str,
        artifacts: ArtifactGetter,
        opts: pulumi.ResourceOptions,
    ) -> None:
        super().__init__("grapl:FirecrackerAssets", name, None, opts)

        child_opts = pulumi.ResourceOptions(parent=self)

        firecracker_kernel_asset = local_or_remote_asset(
            local_path=path_from_root("dist/firecracker_kernel.tar.gz"),
            artifacts=artifacts,
            artifact_key="firecraker_kernel.tar.gz",
            repository_name=repository_name,
        )


def cloudsmith_cdn_uri(
    repository_name: str,
    package_name: str,
    version: str,
) -> str:
    return (
        f"https://dl.cloudsmith.io/public/{repository_name}"
        f"/raw/versions/{version}/{package_name}"
    )


def local_or_remote_asset(
    local_path: Path,
    artifacts: ArtifactGetter,
    artifact_key: str,
    repository_name: Optional[str],
) -> pulumi.asset.Asset:
    # First, allow a local asset if it's local-grapl
    if config.LOCAL_GRAPL:
        if local_path.resolve().exists():
            return pulumi.asset.FileAsset(local_path)

    # Or fall back to a remote path
    version = artifacts.get(artifact_key)
    if version and repository_name:
        uri = cloudsmith_cdn_uri(
            repository_name=repository_name,
            package_name=artifact_key,
            version=version,
        )
        return pulumi.asset.RemoteAsset(uri)

    raise ValueError(
        f"Couldn't find an asset at {local_path}"
        f" or an artifact with key {artifact_key}"
    )
