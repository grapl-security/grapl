from typing import Mapping
import pulumi
from pathlib import Path
from infra import config
from infra.path import path_from_root

class FirecrackerAssets(pulumi.ComponentResource):
    """
    A Postgres instance running in RDS.
    """

    def __init__(
        self,
        name: str,
        repository: str,
        artifacts: Mapping[str, str],
        opts: pulumi.ResourceOptions,
    ) -> None:
        super().__init__("grapl:FirecrackerAssets", name, None, opts)

        child_opts = pulumi.ResourceOptions(parent=self)

        firecracker_package_name = "firecracker_kernel.tar.gz"
        firecracker_kernel_asset = local_or_remote_asset(
            local_path=path_from_root("dist/firecracker_kernel.tar.gz"),
            remote_url=cloudsmith_cdn_url(
                repository=repository,
                package_name=firecracker_package_name,
                version=artifacts.get(firecracker_package_name)
            ),
            opts=child_opts,
        )

def cloudsmith_cdn_url(
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
    remote_url: str,
    opts: pulumi.ResourceOptions,
) -> pulumi.asset.Asset:
    # First, allow a local asset if it's local-grapl
    if config.LOCAL_GRAPL:
        if local_path.resolve().exists():
            return pulumi.asset.FileArchive(local_path, opts=opts)

    # Or fall back to a remote path
    if 
    