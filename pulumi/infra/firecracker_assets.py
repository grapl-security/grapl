from pathlib import Path
from typing import Optional

import pulumi_aws as aws
from infra import config
from infra.artifacts import ArtifactGetter
from infra.bucket import Bucket
from infra.path import path_from_root
from infra.s3_url import get_s3_url

import pulumi

FIRECRACKER_KERNEL_FILENAME = "firecracker_kernel.tar.gz"
FIRECRACKER_ROOTFS_FILENAME = "firecracker_rootfs.tar.gz"


class FirecrackerAssets(pulumi.ComponentResource):
    """
    Uploads Firecracker assets from disk-or-Cloudsmith into
    S3 bucket.

    TODO: In prod, should we serve the assets from S3 or Cloudsmith?
    https://github.com/grapl-security/issue-tracker/issues/857
    """

    def __init__(
        self,
        name: str,
        repository_name: Optional[str],
        artifacts: ArtifactGetter,
        opts: pulumi.ResourceOptions = None,
    ) -> None:
        super().__init__("grapl:FirecrackerAssets", name, None, opts)

        self.kernel_asset = local_or_remote_asset(
            local_path=path_from_root("dist/firecracker_kernel.tar.gz"),
            artifacts=artifacts,
            artifact_key=FIRECRACKER_KERNEL_FILENAME,
            repository_name=repository_name,
        )

        self.rootfs_asset = local_or_remote_asset(
            local_path=path_from_root("dist/firecracker_rootfs.tar.gz"),
            artifacts=artifacts,
            artifact_key=FIRECRACKER_ROOTFS_FILENAME,
            repository_name=repository_name,
        )


class FirecrackerS3BucketObjects(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        plugins_bucket: Bucket,
        firecracker_assets: FirecrackerAssets,
        opts: pulumi.ResourceOptions = None,
    ) -> None:
        super().__init__("grapl:FirecrackerS3BucketObjects", name, None, opts)

        # If we had delete_before_replace=False, then this happens:
        # - Upload new file to BUCKET/KEY
        # - Delete file at BUCKET/KEY
        # - Now we have no kernel
        # It's still not perfect because there is still a period
        # where there is no kernel available; anything that spins
        # up in those few seconds is going to have a bad time.
        delete_before_replace = True

        kernel_s3obj = aws.s3.BucketObject(
            "firecracker_kernel",
            key=FIRECRACKER_KERNEL_FILENAME,
            bucket=plugins_bucket.bucket,
            source=firecracker_assets.kernel_asset,
            opts=pulumi.ResourceOptions(
                delete_before_replace=delete_before_replace,
                parent=self,
            ),
        )
        self.kernel_s3obj_url = get_s3_url(kernel_s3obj)

        rootfs_s3obj = aws.s3.BucketObject(
            "firecracker_rootfs",
            key=FIRECRACKER_ROOTFS_FILENAME,
            bucket=plugins_bucket.bucket,
            source=firecracker_assets.rootfs_asset,
            opts=pulumi.ResourceOptions(
                delete_before_replace=delete_before_replace,
                parent=self,
            ),
        )
        self.rootfs_s3obj_url = get_s3_url(rootfs_s3obj)


def cloudsmith_cdn_url(
    repository_name: str,
    package_name: str,
    version: str,
) -> str:
    # NOTE: the `raw` in here is a Cloudsmith package type
    # (to be contrasted with 'python package' or 'docker image')
    # nothing to do with the `grapl/raw` repository.
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
    # Check Pulumi.stackname.yaml for a Cloudsmith-hosted asset
    version = artifacts.get(artifact_key)
    if version and repository_name:
        url = cloudsmith_cdn_url(
            repository_name=repository_name,
            package_name=artifact_key,
            version=version,
        )
        return pulumi.asset.RemoteAsset(url)

    # Allow a local asset if it's local-grapl
    if config.LOCAL_GRAPL:
        if local_path.resolve().exists():
            return pulumi.asset.FileAsset(local_path)

    raise ValueError(
        f"Couldn't find an asset at {local_path} or an artifact with "
        f"key {artifact_key} in `{config.STACK_CONFIG_FILENAME}`."
    )
