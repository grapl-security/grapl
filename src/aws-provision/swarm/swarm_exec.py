import base64
import subprocess
import sys
from typing import List


def _exec(command: List[str]) -> str:
    """Run the given command. Returns the result."""
    result = subprocess.run(command, check=True, capture_output=True)
    return result.stdout.decode("utf-8")


def main(raw_command: str) -> None:
    command = base64.b64decode(bytes(raw_command, "utf-8")).decode("utf-8").split(",")
    sys.stdout.write(_exec(command))


if __name__ == "__main__":
    main(sys.argv[1])
