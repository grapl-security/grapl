import subprocess
import sys


def _docker_swarm_join_token(manager: bool) -> str:
    """Return the swarm join token. If manager is True, return a manager
    token, otherwise return a worker token."""
    result = subprocess.run(
        ["docker", "swarm", "join-token", "manager" if manager else "worker", "-q"],
        check=True,
        capture_output=True,
    )
    return result.stdout.decode("utf-8")


def main(manager: bool) -> None:
    sys.stdout.write(_docker_swarm_join_token(manager))


if __name__ == "__main__":
    main(sys.argv[1] == "true")
