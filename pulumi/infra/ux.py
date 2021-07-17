from pathlib import Path

from infra.bucket import Bucket
from infra.config import configured_version_for, repository_path


def _ux_source_directory() -> Path:
    # If we have a pinned version for the grapl-ux set in our stack
    # configuration, we should have downloaded the artifact from the
    # artifact repository and unpacked it into a version-tagged
    # directory for us to pull from.
    #
    # We have to do this because there isn't an apparent way in
    # Pulumi to expand a tarball into an S3 bucket. If that every
    # becomes possible, we should us it, by all means!
    #
    # If there's no configured version, we just pull from the
    # build directory in the local source tree (it will need to
    # have been built first, too).
    grapl_ux_version = configured_version_for("grapl-ux")
    directory = (
        f"dist/grapl-ux-{grapl_ux_version}"
        if grapl_ux_version
        else "src/js/engagement_view/build"
    )
    return repository_path(directory).resolve()


def populate_ux_bucket(ux_bucket: Bucket) -> None:
    source_dir = _ux_source_directory()

    try:
        ux_bucket.upload_to_bucket(source_dir, root_path=source_dir)
    except FileNotFoundError as e:
        raise Exception(
            """
            You need to either build or download UX assets first.

            If you have a pinned version for `grapl-ux` in your stack configuration, please run

                pulumi/bin/prepare_grapl_ux_depencency.sh grapl/<STACK>

            If you do *not* have a pinned version in your stack configuration, you can run the above, or

                make build-ux
            """
        ) from e
