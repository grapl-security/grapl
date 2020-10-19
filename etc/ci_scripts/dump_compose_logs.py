#!/usr/bin/python3

from typing import Any, List
from pathlib import Path
import os
import subprocess
import sys

# need minimum 3.7 for capture_output=True
assert sys.version_info >= (3, 7)


def _name_of_all_containers(compose_project: str) -> List[str]:
    """
    compose_project meaning the `project: <thing>` in the `compose=` section
    of your dobi.yaml
    """
    run_result = subprocess.run(
        [
            "docker",
            "ps",
            "--all",
            "--filter",
            f"name={compose_project}",
            "--format",
            "table {{.Names}}",
        ],
        capture_output=True,
    )
    containers: List[str] = run_result.stdout.decode("utf-8").split("\n")
    containers = containers[1:]  # remove the table column header
    containers = [c for c in containers if c]  # filter empty
    if not containers:
        raise ValueError(f"Couldn't find any containers for '{compose_project}'")
    return containers


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


LOG_ARTIFACTS_PATH = Path("/tmp/log_artifacts").resolve()


def dump_all_logs(compose_project: str) -> None:
    containers = _name_of_all_containers(compose_project)
    os.makedirs(LOG_ARTIFACTS_PATH, exist_ok=True)
    for container in containers:
        _dump_docker_log(container_name=container, dir=LOG_ARTIFACTS_PATH)


def parse_args() -> Any:
    import argparse

    parser = argparse.ArgumentParser(
        description="Dump all Docker logs for a given Dobi Compose"
    )
    parser.add_argument("--compose-project", dest="compose_project", required=True)
    return parser.parse_args()


if __name__ == "__main__":
    args = parse_args()
    dump_all_logs(compose_project=args.compose_project)
