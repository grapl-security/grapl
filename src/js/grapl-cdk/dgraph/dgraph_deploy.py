import subprocess
import sys

from typing import Tuple, Iterator


def _deploy_dgraph(
    prefix: str,
    manager_hostname: str,
    worker_hostnames: Tuple[str, str],
) -> Iterator[str]:
    """Deploy DGraph on a docker swarm cluster"""
    commands = [
        ["sudo", "su", "ec2-user"],
        ["cd", "$HOME"],
        ["export", f"GRAPL_DEPLOY_NAME={prefix}"],
        ["export", f"AWS_LOGS_GROUP={prefix}-grapl-dgraph"],
        ["export", f"AWS01_NAME={manager_hostname}"],
        ["export", f"AWS02_NAME={worker_hostnames[0]}"],
        ["export", f"AWS03_NAME={worker_hostnames[1]}"],
        [
            "aws",
            "s3",
            "cp",
            f"s3://${{GRAPL_DEPLOY_NAME,,}}-dgraph-config-bucket/docker-compose-dgraph.yml",
            ".",
        ],
        [
            "aws",
            "s3",
            "cp",
            f"s3://${{GRAPL_DEPLOY_NAME,,}}-dgraph-config-bucket/envoy.yaml",
            ".",
        ],
        ["docker", "stack", "deploy", "-c", "docker-compose-dgraph.yml", "dgraph"],
    ]
    for command in commands:
        result = subprocess.run(command, check=True, capture_output=True)
        yield result.stdout.decode("utf-8")


def main(
    prefix: str,
    manager_hostname: str,
    worker_hostnames: Tuple[str, str],
) -> None:
    for result in _deploy_dgraph(prefix, manager_hostname, worker_hostnames):
        sys.stdout.write(result)


if __name__ == "__main__":
    main(
        prefix=sys.argv[1],
        manager_hostname=sys.argv[2],
        worker_hostnames=(sys.argv[3], sys.argv[4]),
    )
