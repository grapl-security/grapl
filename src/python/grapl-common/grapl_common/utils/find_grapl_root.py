import subprocess
from pathlib import Path
from typing import Optional


def find_grapl_root() -> Optional[Path]:
    # We could potentially add other heuristics here.
    return _find_grapl_root_based_off_git_in_pwd()


def _find_grapl_root_based_off_git_in_pwd() -> Optional[Path]:
    cmd = "git rev-parse --show-toplevel"
    git_repo_root = _quietly_execute(cmd)
    if git_repo_root is None:
        return None

    git_repo_root_path = Path(git_repo_root)
    del git_repo_root

    if git_repo_root_path.name == "grapl":
        # It's pretty likely that we've found the grapl root.
        return git_repo_root_path

    return None


def _quietly_execute(cmd: str) -> Optional[str]:
    try:
        return (
            subprocess.check_output(
                cmd,
                shell=True,
                # Swallow the stderr stuff
                stderr=subprocess.DEVNULL,
            )
            .decode("utf-8")
            .strip()
        )
    except subprocess.CalledProcessError:
        # The command failed
        return None
