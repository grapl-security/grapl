import json
import os
import shlex
import subprocess
import sys
from typing import Dict


def _merge_daemon_config(config_update: Dict) -> Dict:
    """merge the given configuration update with the existing
    configuration in /etc/docker/daemon.json

    """
    config = {}
    if os.path.exists("/etc/docker/daemon.json"):
        with open("/etc/docker/daemon.json", "r") as infile:
            config = json.load(infile)
    for k, v in config_update.items():
        config[k] = v
    subprocess.run(
        [
            "sudo",
            "bash",
            "-c",
            " ".join(
                [
                    "echo",
                    shlex.quote(json.dumps(config, separators=(",", ":"))),
                    ">",
                    "/etc/docker/daemon.json",
                ]
            ),
        ],
        check=True,
    )
    return config


def main(raw_config: str) -> None:
    config = _merge_daemon_config(json.loads(raw_config))
    sys.stdout.write(json.dumps(config))


if __name__ == "__main__":
    main(sys.argv[1])
