import subprocess
import sys

from typing import Iterator


def _init_dgraph() -> Iterator[str]:
    """Initialize a DGraph instance. Make sure the instance_init.py
    command has completed before running this command."""
    commands = [
        # create LUKS key
        ["head", "-c", "256", "/dev/urandom", ">", "/root/luks_key"],
        ["cryptsetup", "-v", "-q", "luksFormat", "/dev/nvme0n1", "/root/luks_key"],
        ['UUID=$(lsblk -o +UUID | grep nvme0n1 | rev | cut -d" " -f1 | rev)'],
        [
            "echo",
            "-e",
            '"dgraph\tUUID=$UUID\t/root/luks_key\tnofail"',
            ">",
            "/etc/crypttab",
        ],
        ["systemctl", "daemon-reload"],
        ["systemctl", "start", "systemd-cryptsetup@dgraph.service"],
        # set up the /dgraph partition
        ["mkfs", "-t", "xfs", "/dev/mapper/dgraph"],
        ["mkdir", "/dgraph"],
        [
            "echo",
            "-e",
            '"/dev/mapper/dgraph\t/dgraph\txfs\tdefaults,nofail\t0\t2"',
            ">>",
            "/etc/fstab",
        ],
        ["mount", "/dgraph"],
        ["echo", "-e", '\'{"data-root":"/dgraph"}\'', ">", "/etc/docker/daemon.json"],
        # restart all the daemons
        ["systemctl", "restart", "docker.service"],
        ["amazon-cloudwatch-agent-ctl", "-m", "ec2", "-a", "stop"],
        ["amazon-cloudwatch-agent-ctl", "-m", "ec2", "-a", "start"],
    ]
    for command in commands:
        result = subprocess.run(command, check=True, capture_stdout=True)
        yield result.stdout.decode("utf-8")


def main() -> None:
    # run all the command to initialize the instance
    for result in _init_instance():
        sys.stdout.write(result)


if __name__ == "__main__":
    main()
