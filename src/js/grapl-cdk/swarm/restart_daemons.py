import subprocess
import sys
from typing import Iterator


def _restart_daemons() -> Iterator[str]:
    """Restart the cloudwatch agent and docker daemon"""
    commands = [
        ["sudo", "amazon-cloudwatch-agent-ctl", "-m", "ec2", "-a", "stop"],
        ["sudo", "amazon-cloudwatch-agent-ctl", "-m", "ec2", "-a", "start"],
        ["sudo", "systemctl", "restart", "docker.service"],
    ]
    for command in commands:
        result = subprocess.run(command, check=True, capture_output=True)
        yield result.stdout.decode("utf-8")


def main() -> None:
    for result in _restart_daemons():
        sys.stdout.write(result)


if __name__ == "__main__":
    main()
