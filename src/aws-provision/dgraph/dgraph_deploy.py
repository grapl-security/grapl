import subprocess
import sys
from typing import Iterator, Tuple


def _deploy_dgraph(
    deployment_name: str,
    manager_hostname: str,
    worker_hostnames: Tuple[str, str],
) -> Iterator[str]:
    """Deploy DGraph on a docker swarm cluster"""
    commands = [
        [
            "aws",
            "s3",
            "cp",
            f"s3://{deployment_name.lower()}-dgraph-config-bucket/dgraph_deploy.sh",
            ".",
        ],
        [
            "bash",
            "dgraph_deploy.sh",
            deployment_name.lower(),
            manager_hostname,
            worker_hostnames[0],
            worker_hostnames[1],
        ],
        ["sleep", "15"],
        ["docker", "service", "ls"],
    ]
    for command in commands:
        try:
            result = subprocess.run(command, check=True, capture_output=True)
        except subprocess.CalledProcessError as e:
            # Just make sure we flush both
            sys.stdout.write(e.stdout.decode("utf-8"))
            sys.stderr.write(e.stderr.decode("utf-8"))
            raise e
        yield result.stdout.decode("utf-8")


def main(
    deployment_name: str,
    manager_hostname: str,
    worker_hostnames: Tuple[str, str],
) -> None:
    for result in _deploy_dgraph(deployment_name, manager_hostname, worker_hostnames):
        sys.stdout.write(result)


if __name__ == "__main__":
    main(
        deployment_name=sys.argv[1],
        manager_hostname=sys.argv[2],
        worker_hostnames=(sys.argv[3], sys.argv[4]),
    )
