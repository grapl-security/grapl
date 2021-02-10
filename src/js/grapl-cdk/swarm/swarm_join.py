import subprocess
import sys


def _join_docker_swarm(join_token: str, manager_ip: str) -> str:
    """Join this instance to the swarm"""
    result = subprocess.run(
        ["docker", "swarm", "join", "--token", join_token, f"{manager_ip}:2377"],
        check=True,
        capture_output=True,
    )
    return result.stdout.decode("utf-8")


def main(join_token: str, manager_ip: str) -> None:
    sys.stdout.write(_join_docker_swarm(join_token, manager_ip))


if __name__ == "__main__":
    join_token = sys.argv[1]
    manager_ip = sys.argv[2]
    main(join_token, manager_ip)
