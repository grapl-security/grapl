import subprocess
import sys
from typing import Iterator, Tuple


def _deploy_dgraph(
    manager_hostname: str,
    worker_hostnames: Tuple[str, str],
    dgraph_config_bucket: str,
    dgraph_logs_group: str,
) -> Iterator[str]:
    """Deploy DGraph on a docker swarm cluster"""

    commands = [
        [
            "aws",
            "s3",
            "cp",
            f"s3://{dgraph_config_bucket}/dgraph_deploy.sh",
            ".",
        ],
        [
            "bash",
            "dgraph_deploy.sh",
            manager_hostname,
            worker_hostnames[0],
            worker_hostnames[1],
            dgraph_config_bucket,
            dgraph_logs_group,
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
    manager_hostname: str,
    worker_hostnames: Tuple[str, str],
    dgraph_config_bucket: str,
    dgraph_logs_group: str,
) -> None:
    for result in _deploy_dgraph(
        manager_hostname,
        worker_hostnames,
        dgraph_config_bucket,
        dgraph_logs_group,
    ):
        sys.stdout.write(result)


if __name__ == "__main__":
    main(
        manager_hostname=sys.argv[1],
        worker_hostnames=(sys.argv[2], sys.argv[3]),
        dgraph_config_bucket=sys.argv[4],
        dgraph_logs_group=sys.argv[5],
    )
