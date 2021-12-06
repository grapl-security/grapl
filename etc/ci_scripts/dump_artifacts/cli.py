#!/usr/bin/env python3

import logging
import os
import sys
from datetime import datetime
from pathlib import Path
from typing import Any, Optional

# Odd path is due to the `/etc` root pattern in pants.toml, fyi
from ci_scripts.dump_artifacts import docker_artifacts, nomad_artifacts

# need minimum 3.7 for capture_output=True
assert sys.version_info >= (
    3,
    7,
), f"Expected version info to be gt, but was {sys.version_info}"

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(logging.INFO)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))


def parse_args() -> Any:
    import argparse

    parser = argparse.ArgumentParser(
        description="Dump all Docker logs for a given docker-compose project"
    )
    parser.add_argument(
        "--compose-project",
        dest="compose_project",
        required=False,
        default=None,
        help="Docker Compose project. Do not specify if Docker-Compose is not involved (e.g. running against prod)",
    )
    parser.add_argument(
        "--dump-agent-logs",
        dest="dump_agent_logs",
        required=False,
        default=False,
        help="Dump the logs for Nomad/Consul agents (e.g. running locally)",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    compose_project: Optional[str] = args.compose_project
    dump_agent_logs: bool = args.dump_agent_logs

    cwd = os.getcwd()
    timestamp = datetime.now().strftime("%Y%m%d%H%M%S")
    artifacts_dir = Path(f"{cwd}/test_artifacts/{compose_project}_{timestamp}")
    os.makedirs(artifacts_dir, exist_ok=False)

    if compose_project:
        docker_artifacts.dump_all_docker_logs(
            compose_project=compose_project, artifacts_dir=artifacts_dir
        )
        docker_artifacts.dump_volume(
            compose_project=compose_project,
            volume_name="dgraph_export",
            artifacts_dir=artifacts_dir,
        )

    # dynamodb dump is done in the e2e binary, which is outside compose - hence, no compose project.
    docker_artifacts.dump_volume(
        compose_project=None, volume_name="dynamodb_dump", artifacts_dir=artifacts_dir
    )

    nomad_artifacts.dump_all(artifacts_dir, dump_agent_logs=dump_agent_logs)
    LOGGER.info(f"Dumped to {artifacts_dir}")


if __name__ == "__main__":
    main()
