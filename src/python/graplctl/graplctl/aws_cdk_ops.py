import logging
import os
import pathlib
import shutil
import subprocess
import sys
from typing import IO, AnyStr

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(os.getenv("GRAPL_LOG_LEVEL", "INFO"))
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))


def deploy_grapl(
    grapl_root: pathlib.Path, aws_profile: str, stdout: IO[AnyStr], stderr: IO[AnyStr]
) -> None:
    grapl_cdk_dir = pathlib.Path(grapl_root.absolute(), "src/js/grapl-cdk")
    edge_ux_artifact_dir = pathlib.Path(grapl_cdk_dir, "/edge_ux_post_replace")
    outputs_file = pathlib.Path(grapl_cdk_dir, "cdk-output.json")

    if edge_ux_artifact_dir.exists():
        shutil.rmtree(edge_ux_artifact_dir)

    os.mkdir(edge_ux_artifact_dir)

    LOGGER.info("building cdk")
    subprocess.run(
        f"cd {grapl_cdk_dir.as_posix()} && npm run build",
        stdout=stdout,
        stderr=stderr,
        check=True,
        shell=True,
    )
    LOGGER.info("built cdk")

    LOGGER.info("deploying Grapl stack")
    subprocess.run(
        f"cd {grapl_cdk_dir.as_posix()} && AWS_PROFILE={aws_profile} cdk deploy --require-approval=never --profile={aws_profile} --outputs-file={outputs_file.as_posix()} Grapl",
        stdout=stdout,
        stderr=stderr,
        check=True,
        shell=True,
    )
    LOGGER.info("deployed Grapl stack")

    shutil.rmtree(edge_ux_artifact_dir)
    os.mkdir(edge_ux_artifact_dir)

    LOGGER.info("creating edge UX package")
    subprocess.run(
        f"cd {grapl_cdk_dir.as_posix()} && npm run create_edge_ux_package",
        stdout=stdout,
        stderr=stderr,
        check=True,
        shell=True,
    )
    LOGGER.info("created edge UX package")

    LOGGER.info("deploying EngagementUX stack")
    subprocess.run(
        f"cd {grapl_cdk_dir.as_posix()} && AWS_PROFILE={aws_profile} cdk deploy --require-approval=never --profile={aws_profile} --outputs-file={outputs_file.as_posix()} EngagementUX",
        stdout=stdout,
        stderr=stderr,
        check=True,
        shell=True,
    )
    LOGGER.info("deployed EngagementUX stack")

    shutil.rmtree(edge_ux_artifact_dir)


def destroy_grapl(
    grapl_root: pathlib.Path, aws_profile: str, stdout: IO[AnyStr], stderr: IO[AnyStr]
) -> None:
    grapl_cdk_dir = pathlib.Path(grapl_root.absolute(), "src/js/grapl-cdk")

    LOGGER.info("building cdk")
    subprocess.run(
        f"cd {grapl_cdk_dir.as_posix()} && npm run build",
        stdout=stdout,
        stderr=stderr,
        check=True,
        shell=True,
    )
    LOGGER.info("built cdk")

    LOGGER.info("destroying all stacks")
    subprocess.run(
        f'cd {grapl_cdk_dir.as_posix()} && AWS_PROFILE={aws_profile} cdk destroy --force --require-approval=never "*"',
        stdout=stdout,
        stderr=stderr,
        check=True,
        shell=True,
    )
    LOGGER.info("destroyed all stacks")


def provision_grapl() -> None:
    pass  # FIXME
