import subprocess
import sys
import urllib.request


def _get_private_ip() -> str:
    return (
        urllib.request.urlopen(
            "curl http://169.254.169.254/latest/meta-data/local-ipv4"
        )
        .read()
        .decode()
    )


def _init_docker_swarm(private_ip: str) -> str:
    """Initialize the docker swarm manager. Returns the join token
    necessary to attach workers to the swarm.

    """
    subprocess.run(
        ["docker", "swarm", "init", "--advertise-addr", private_ip], check=True
    )
    result = subprocess.run(
        ["docker", "swarm", "join-token", "worker", "-q"],
        check=True,
        capture_output=True,
    )
    return result.stdout


def main() -> None:
    sys.stdout.write(_init_docker_swarm(private_ip=_get_private_ip()))


if __name__ == "__main__":
    main()
