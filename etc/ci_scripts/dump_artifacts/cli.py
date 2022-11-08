#!/usr/bin/env python3

import argparse
import logging
import os
import shutil
import sys
from datetime import datetime
from pathlib import Path

# Odd path is due to the `/etc` root pattern in pants.toml, fyi
from ci_scripts.dump_artifacts import docker_artifacts, nomad_artifacts
from ci_scripts.dump_artifacts.colorize import Colorize

# need minimum 3.7 for capture_output=True
assert sys.version_info >= (
    3,
    7,
), f"Expected version info to be gt, but was {sys.version_info}"

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(logging.INFO)
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))


class Args:
    def __init__(self) -> None:
        parser = argparse.ArgumentParser(
            description="Dump all Docker logs for a given docker compose project"
        )
        parser.add_argument(
            "--compose-project",
            dest="compose_project",
            required=False,
            default=None,
            help="Docker Compose project. Do not specify if Docker Compose is not involved (e.g. running against prod)",
        )
        parser.add_argument(
            "--dump-agent-logs",
            dest="dump_agent_logs",
            action="store_true",
            help="Dump the logs for Nomad/Consul agents (useful if running locally)",
        )
        parser.add_argument(
            "--no-dump-agent-logs", dest="dump_agent_logs", action="store_false"
        )
        parser.add_argument(
            "--dump-connect-proxy-logs",
            dest="dump_connect_proxy_logs",
            action="store_true",
            help="Dump the logs for Consul Connect",
        )
        parser.add_argument(
            "--no-dump-connect-proxy-logs",
            dest="dump_connect_proxy_logs",
            action="store_false",
        )
        parser.set_defaults(dump_agent_logs=False, dump_connect_proxy_logs=False)
        args = parser.parse_args()
        self.compose_project: str | None = args.compose_project
        self.dump_agent_logs: bool = args.dump_agent_logs
        self.dump_connect_proxy_logs: bool = args.dump_connect_proxy_logs


def main() -> None:
    args = Args()
    compose_project = args.compose_project

    cwd = os.getcwd()
    timestamp = datetime.now().isoformat(timespec="seconds")
    timestamp = timestamp.replace(":", "-")  # colons in paths are bad
    artifacts_dir = Path(f"{cwd}/test_artifacts/{timestamp}")
    os.makedirs(artifacts_dir, exist_ok=False)

    if compose_project:
        docker_artifacts.dump_all_docker_logs(
            compose_project=compose_project, artifacts_dir=artifacts_dir
        )

    # dynamodb dump is done in the e2e binary, which is outside compose - hence, no compose project.
    docker_artifacts.dump_volume(
        compose_project=None, volume_name="dynamodb_dump", artifacts_dir=artifacts_dir
    )

    # Dump metrics and traces to files
    docker_artifacts.write_otel_files(artifacts_dir)

    nomad_dump_options = nomad_artifacts.NomadDumpOptions(
        dump_agent_logs=args.dump_agent_logs,
        dump_connect_proxy_logs=args.dump_connect_proxy_logs,
    )
    nomad_artifacts.dump_all(
        artifacts_dir,
        opts=nomad_dump_options,
    )

    # Zip up everything. We can't zip up directly into artifacts_dir or you
    # get a recursive zip - that is to say, eating up all the space on disk.
    zip_filename = "/tmp/ALL_ARTIFACTS"
    shutil.make_archive(base_name=zip_filename, format="zip", root_dir=artifacts_dir)
    shutil.move(src="/tmp/ALL_ARTIFACTS.zip", dst=artifacts_dir)

    LOGGER.info(
        "\n".join(
            [
                "---",
                f"--- Artifacts dumped to {Colorize.green(str(artifacts_dir))}",
                "---",
            ]
        )
    )


if __name__ == "__main__":
    main()
