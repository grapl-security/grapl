#!/usr/bin/env python3

import logging
import subprocess
import sys
from pathlib import Path
from typing import List, Optional

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(logging.INFO)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))


def _container_names_by_prefix(prefix: str) -> List[str]:
    """Return a list of all containers (running or not) whose names begin
    with `prefix`.

    Provide a `docker-compose` project name as a prefix to retrieve
    containers associated with that project. (We don't use
    `docker-compose` to do this directly because of our rather complex
    usage of multiple compose files at a time.)

    Raises an error if no such containers are found.

    """
    result = subprocess.run(
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
        text=True,
    )
    containers = result.stdout.split()
    if not containers:
        LOGGER.warning(
            f"Couldn't find any containers for '{prefix}'; it's possible you"
            " got a cancellation signal between bringing up Nomad and"
            " deploying Pulumi."
        )
        raise ValueError()
    return containers


def dump_docker_ps(dir: Path) -> None:
    """
    run `docker ps` and dump to $DIR/docker_ps.log
    """
    destination = dir / "docker_ps.log"
    LOGGER.debug(f"Dumping 'docker ps' to '{destination}'")
    with open(destination, "wb") as out_stream:
        subprocess.run(
            # --all includes containers that have already exited
            f"docker ps --all",
            stdout=out_stream,
            stderr=out_stream,
            shell=True,
        )


def _dump_docker_log(container_name: str, dir: Path) -> None:
    """
    run `docker logs` and dump to $DIR/$CONTAINER_NAME.log
    """
    destination = dir / f"{container_name}.log"
    LOGGER.debug(f"Dumping logs for '{container_name}' container to '{destination}'")
    with open(destination, "wb") as out_stream:
        subprocess.run(
            f"docker logs --timestamps {container_name}",
            stdout=out_stream,
            stderr=out_stream,
            shell=True,
        )


def dump_volume(
    compose_project: Optional[str], volume_name: str, artifacts_dir: Path
) -> None:
    # Make a temporary container with the volume mounted
    # docker-compose prefixes volume names with the compose project name.
    prefix = f"{compose_project}_" if compose_project else ""
    container_id = subprocess.run(
        f"docker run -d --volume {prefix}{volume_name}:/{volume_name} alpine true",
        shell=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    LOGGER.debug(f"Temporary container {container_id}")

    # Copy contents of /mounted_volume into artifacts_dir
    subprocess.run(
        f"docker cp {container_id}:/{volume_name} {artifacts_dir}", shell=True
    )

    subprocess.run(f"docker rm {container_id}", shell=True)


def dump_all_docker_logs(compose_project: str, artifacts_dir: Path) -> None:
    dump_docker_ps(artifacts_dir)

    containers = _container_names_by_prefix(compose_project)
    for container in containers:
        _dump_docker_log(container_name=container, dir=artifacts_dir)
