import subprocess
import sys
from typing import Iterator


def _init_dgraph(dgraph_config_bucket: str) -> Iterator[str]:
    """Initialize a DGraph instance. Make sure the instance_init.py
    command has completed before running this command."""
    commands = [
        [
            "aws",
            "s3",
            "cp",
            f"s3://{dgraph_config_bucket}/dgraph_init.sh",
            ".",
        ],
        ["bash", "dgraph_init.sh"],
    ]
    for command in commands:
        result = subprocess.run(command, check=True, capture_output=True)
        yield result.stdout.decode("utf-8")


def main(dgraph_config_bucket: str) -> None:
    # run all the command to initialize the instance
    for result in _init_dgraph(dgraph_config_bucket):
        sys.stdout.write(result)


if __name__ == "__main__":
    main(sys.argv[1])
