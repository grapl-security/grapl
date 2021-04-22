#!/usr/bin/env python

import logging
import os
import subprocess
import sys
from pathlib import Path
from typing import Any, List, Optional

# need minimum 3.7 for capture_output=True
assert sys.version_info >= (
    3,
    7,
), f"Expected version info to be gt, but was {sys.version_info}"

logging.basicConfig(stream=sys.stdout, level=logging.INFO)


def _container_names_by_prefix(prefix: str) -> List[str]:
    """Return a list of all containers (running or not) whose names begin
    with `prefix`.

    Provide a `docker-compose` project name as a prefix to retrieve
    containers associated with that project. (We don't use
    `docker-compose` to do this directly because of our rather complex
    usage of multiple compose files at a time.)

    Raises an error if no such containers are found.

    """
    run_result = subprocess.run(
        [
            "docker",
            "ps",
            "--all",
            "--filter",
            f"name={prefix}",
            "--format",
            "{{.Names}}",
        ],
        capture_output=True,
    )
    containers: List[str] = run_result.stdout.decode("utf-8").split("\n")
    containers = [c for c in containers if c]  # filter empty
    if not containers:
        raise ValueError(f"Couldn't find any containers for '{prefix}'")
    return containers


def _lambda_names() -> List[str]:
    """
    Return the names of the lambdas that are running in our Localstack instance.
    """

    # All Localstack lambda containers begin with this prefix; what
    # remains is the actual lambda's name.
    #
    # We always use the us-east-1 region, and don't change the account
    # ID from Localstack's default of 000000000000; if that ever
    # changes, this prefix would need to be changed accordingly.
    lambda_container_prefix="localstack_lambda_arn_aws_lambda_us-east-1_000000000000_function_"
    containers=_container_names_by_prefix(lambda_container_prefix)

    # Chop off the prefix to get just the names of the lambda functions
    return [name.replace(lambda_container_prefix, "") for name in containers ]


def _dump_lambda_log(lambda_name: str, dir: Path) -> None:
    """Dump the Cloudwatch logs of the given lambda function from Localstack.

    Requires the AWS CLI to be present, and assumes a running
    Localstack instance is available at http://localhost:4566.

    These same logs are also available in the Localstack container's
    log output, but mixed in with the logs of every other lambda. By
    using this function, we can isolate them.

    Note, however, that these logs will be only what the lambda
    function itself outputs during its execution. There will still be
    important information in the Localstack logs concerning how the
    functions are *invoked* that may be useful in debugging issues.

    """
    destination = dir / f"{lambda_name}_lambda.log"
    logging.info(f"Dumping logs for '{lambda_name}' lambda function to '{destination}'")
    with open(destination, "wb") as out_stream:
        subprocess.run(
            [
                "aws",
                "--endpoint-url=http://localhost:4566",
                "logs",
                "tail",
                f"/aws/lambda/{lambda_name}"
            ],
            stdout=out_stream,
            env={
                "PATH": os.environ["PATH"],
                # "test" is the value assumed by Localstack
                "AWS_ACCESS_KEY_ID": "test",
                "AWS_SECRET_ACCESS_KEY": "test"
            }
        )


def _dump_docker_log(container_name: str, dir: Path) -> None:
    """
    run `docker logs` and dump to $DIR/$CONTAINER_NAME.log
    """
    destination = dir / f"{container_name}.log"
    with open(destination, "wb") as out_stream:
        popen = subprocess.Popen(
            [
                "docker",
                "logs",
                "--timestamps",
                container_name,
            ],
            stdout=out_stream,
        )
        popen.wait()


ARTIFACTS_PATH = Path("/tmp/dumped_artifacts").resolve()


def dump_all_logs(compose_project: str) -> None:
    containers = _container_names_by_prefix(compose_project)
    os.makedirs(ARTIFACTS_PATH, exist_ok=True)
    for container in containers:
        _dump_docker_log(container_name=container, dir=ARTIFACTS_PATH)
    for lambda_fn in _lambda_names():
        _dump_lambda_log(lambda_fn, dir=ARTIFACTS_PATH)


def dump_volume(compose_project: Optional[str], volume_name: str) -> None:
    # Make a temporary container with the volume mounted
    # docker-compose prefixes volume names with the compose project name.
    prefix = f"{compose_project}_" if compose_project else ""
    cmd = f"docker run -d --volume {prefix}{volume_name}:/{volume_name} alpine true"
    container_id = (
        subprocess.run(cmd.split(" "), capture_output=True)
        .stdout.decode("utf-8")
        .strip()
    )
    print(f"Temporary container {container_id}")

    os.makedirs(ARTIFACTS_PATH, exist_ok=True)
    # Copy contents of /mounted_volume into ARTIFACTS_PATH
    subprocess.run(
        f"docker cp {container_id}:/{volume_name} {ARTIFACTS_PATH}".split(" "),
    )

    subprocess.run(f"docker rm {container_id}".split(" "))


def parse_args() -> Any:
    import argparse

    parser = argparse.ArgumentParser(
        description="Dump all Docker logs for a given docker-compose project"
    )
    parser.add_argument("--compose-project", dest="compose_project", required=True)
    return parser.parse_args()


if __name__ == "__main__":
    args = parse_args()
    dump_all_logs(compose_project=args.compose_project)
    dump_volume(compose_project=args.compose_project, volume_name="dgraph_export")
    # dynamodb dump is done in the e2e binary, which is outside compose - hence, no compose project.
    dump_volume(compose_project=None, volume_name="dynamodb_dump")
    logging.info(f"Dumped to {ARTIFACTS_PATH}")
