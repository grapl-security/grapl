import base64
import logging
import os
import pathlib
import shutil
import subprocess
import sys
from typing import IO, AnyStr

from mypy_boto3_lambda import LambdaClient

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(os.getenv("GRAPL_LOG_LEVEL", "INFO"))
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

GRAPL_CDK_RELATIVE_PATH = "src/js/grapl-cdk"
EDGE_UX_DIRECTORY = "/edge_ux_post_replace"
CDK_OUT_FILENAME = "cdk-output.json"


def _run_shell_cmd(
    cmd: str,
    cwd: pathlib.Path,
    stdout: IO[AnyStr],
    stderr: IO[AnyStr],
) -> subprocess.CompletedProcess:
    return subprocess.run(
        cmd,
        stdout=stdout,
        stderr=stderr,
        check=True,
        shell=True,
        cwd=cwd.as_posix(),
        executable="/bin/bash",
    )


def deploy_grapl(
    grapl_root: pathlib.Path, aws_profile: str, stdout: IO[AnyStr], stderr: IO[AnyStr]
) -> None:
    grapl_cdk_dir = pathlib.Path(grapl_root.absolute(), GRAPL_CDK_RELATIVE_PATH)
    edge_ux_artifact_dir = pathlib.Path(grapl_cdk_dir, EDGE_UX_DIRECTORY)
    outputs_file = pathlib.Path(grapl_cdk_dir, CDK_OUT_FILENAME)

    if edge_ux_artifact_dir.exists():
        shutil.rmtree(edge_ux_artifact_dir)

    os.mkdir(edge_ux_artifact_dir)

    LOGGER.info("building cdk")
    _run_shell_cmd("npm run build", cwd=grapl_cdk_dir, stdout=stdout, stderr=stderr)
    LOGGER.info("built cdk")

    LOGGER.info("deploying Grapl stack")
    _run_shell_cmd(
        f"cdk deploy --require-approval=never --profile={aws_profile} --outputs-file={outputs_file.as_posix()} Grapl",
        cwd=grapl_cdk_dir,
        stdout=stdout,
        stderr=stderr,
    )
    LOGGER.info("deployed Grapl stack")

    shutil.rmtree(edge_ux_artifact_dir)
    os.mkdir(edge_ux_artifact_dir)

    LOGGER.info("creating edge UX package")
    _run_shell_cmd(
        "npm run create_edge_ux_package",
        cwd=grapl_cdk_dir,
        stdout=stdout,
        stderr=stderr,
    )
    LOGGER.info("created edge UX package")

    LOGGER.info("deploying EngagementUX stack")
    _run_shell_cmd(
        f"cdk deploy --require-approval=never --profile={aws_profile} --outputs-file={outputs_file.as_posix()} EngagementUX",
        cwd=grapl_cdk_dir,
        stdout=stdout,
        stderr=stderr,
    )
    LOGGER.info("deployed EngagementUX stack")

    shutil.rmtree(edge_ux_artifact_dir)


def destroy_grapl(
    grapl_root: pathlib.Path, aws_profile: str, stdout: IO[AnyStr], stderr: IO[AnyStr]
) -> None:
    grapl_cdk_dir = pathlib.Path(grapl_root.absolute(), GRAPL_CDK_RELATIVE_PATH)

    LOGGER.info("building cdk")
    _run_shell_cmd("npm run build", cwd=grapl_cdk_dir, stdout=stdout, stderr=stderr)
    LOGGER.info("built cdk")

    LOGGER.info("destroying all stacks")
    _run_shell_cmd(
        f'cdk destroy --profile={aws_profile} --force --require-approval=never "*"',
        cwd=grapl_cdk_dir,
        stdout=stdout,
        stderr=stderr,
    )
    LOGGER.info("destroyed all stacks")


def _invoke_lambda(lambda_: LambdaClient, function_name: str) -> None:
    LOGGER.info(f"invoking lambda {function_name}")
    result = lambda_.invoke(
        FunctionName=function_name,
        InvocationType="RequestResponse",
        LogType="Tail",
    )

    status = result["StatusCode"]
    logs = base64.b64decode(bytes(result["LogResult"], "utf-8")).decode("utf-8")
    if status == 200 and result.get("FunctionError") is None:
        for line in logs.splitlines():
            LOGGER.info(line)
        LOGGER.info(f"lambda invocation succeeded for {function_name}")
    else:
        LOGGER.error(
            f"{''.join(l.decode('utf-8') for l in result['Payload'].readlines())}"
        )
        raise Exception(
            f"lambda invocation for {function_name} failed with status {status}: {result['FunctionError']}"
        )


def provision_grapl(lambda_: LambdaClient, deployment_name: str) -> None:
    _invoke_lambda(lambda_=lambda_, function_name=f"{deployment_name}-provisioner")


def run_e2e_tests(lambda_: LambdaClient, deployment_name: str) -> None:
    _invoke_lambda(lambda_=lambda_, function_name=f"{deployment_name}-e2e-test-runner")
