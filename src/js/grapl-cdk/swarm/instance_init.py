import subprocess
import sys
from typing import Iterator


def _init_instance() -> Iterator[str]:
    """Initialize a docker swarm instance. Yields log output of each
    command.

    """
    commands = [
        # install CWAgent and docker
        ["yum", "install", "-y", "docker", "amazon-cloudwatch-agent"],
        # add ec2-user to the docker group
        ["usermod", "-a", "-G", "docker", "ec2-user"],
        # start all the daemons
        ["amazon-cloudwatch-agent-ctl", "-m", "ec2", "-a", "start"],
        ["systemctl", "enable", "docker.service"],
        ["systemctl", "start", "docker.service"],
    ]
    for command in commands:
        result = subprocess.run(command, check=True, capture_output=True)
        yield result.stdout.decode("utf-8")


def main() -> None:
    for result in _init_instance():
        sys.stdout.write(result)


if __name__ == "__main__":
    main()
