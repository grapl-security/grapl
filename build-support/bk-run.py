import re
import shutil
import subprocess
import sys
from os import path
from typing import Final, Optional

import typer
from rich.console import Console

DEFAULT_BUILDKITE_ORGANIZATION: Final[str] = "grapl"
DEFAULT_PIPELINE_STAGE: Final[str] = "verify"

err_console = Console(stderr=True)
app = typer.Typer(add_completion=False, rich_markup_mode="rich")


def _ensure_bk() -> None:
    """
    Ensure that the `bk` CLI tool is present; if it's not, there's not much we can do!
    """
    if not shutil.which("bk"):
        err_console.print(
            f":cross_mark: Cannot find [bold white]bk[/bold white] in $PATH. Download it from https://github.com/buildkite/cli/releases. Be sure to run [bold white]bk configure[/bold white]!"
        )
        sys.exit(1)

    bk_config_file = "~/.buildkite/config.json"
    if not path.exists(path.expanduser(bk_config_file)):
        err_console.print(
            f":cross_mark: Cannot find [bold white]{bk_config_file}[/bold white]. Please run [bold white]bk configure[/bold white] first."
        )
        sys.exit(1)


def _repository_name() -> str:
    """Extract the repository name from the Git remote URL. Assumes the
    remote is named "origin".

    """
    result = subprocess.run(
        ["git", "config", "remote.origin.url"],
        capture_output=True,
        text=True,
        check=True,
    )
    url = result.stdout
    # Strip off any ".git" suffix; this simplifies the subsequent
    # regular expression
    url = re.sub(r"\.git$", "", url)
    regex = re.compile("^(git@github.com:|https://github.com/)([^/]+)/(?P<repo>.+)")
    if match := regex.search(url):
        return match["repo"]
    else:
        raise Exception("Could not extract repository name!")


def _message(ref: str) -> str:
    """
    Extract the commit title of the given reference to use as a pipeline message.
    """
    result = subprocess.run(
        ["git", "show", "--pretty=format:%s", "--no-patch", ref],
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout


def _current_branch() -> str:
    """
    Determine the name of the currently checked-out Git branch.
    """
    result = subprocess.run(
        ["git", "rev-parse", "--abbrev-ref", "HEAD"],
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout.strip()


def _push(commit: str, branch: str) -> None:
    """
    Force-push COMMIT to BRANCH on the `origin` upstream.
    """
    command = [
        "git",
        "push",
        "--set-upstream",
        "--force-with-lease",
        "origin",
        f"{commit}:{branch}",
    ]
    err_console.print(f"Executing: [bold white]{' '.join(command)}[/bold white]")
    subprocess.run(command, check=True, capture_output=True)


def _trigger(pipeline: str, branch: str, commit: str, message: str) -> None:
    """
    Trigger a Buildkite pipeline run using the `bk` CLI tool.
    """
    command = [
        "bk",
        "build",
        "create",
        "--pipeline",
        pipeline,
        "--branch",
        branch,
        "--commit",
        commit,
        "--message",
        message,
    ]
    err_console.print(f"Executing: [bold white]{' '.join(command)}[/bold white]")
    subprocess.run(command, check=True, capture_output=True)


@app.command()
def main(
    commit: str = typer.Option("HEAD", help="The commit to execute in the pipeline"),
    branch: Optional[str] = typer.Option(
        None,
        help="The branch on which to execute the pipeline. If not specified, uses the current local branch.",
    ),
    message: Optional[str] = typer.Option(
        None,
        help="The message to use for the pipeline execution. If not specified, uses the specified commit title.",
    ),
    push: bool = typer.Option(
        False,
        help="Whether or not to push the specified commit/branch to before triggering a pipeline",
    ),
    buildkite_organization: str = typer.Option(
        DEFAULT_BUILDKITE_ORGANIZATION,
        help="The Buildkite organization to interact with.",
    ),
    pipeline_stage: str = typer.Option(
        DEFAULT_PIPELINE_STAGE, help="The pipeline stage to trigger."
    ),
) -> None:
    """Trigger a Buildkite pipeline run.

    By default, triggers a pipeline run based on the current branch
    and commit. You can pass the [bold white]--push[/bold white]
    option to perform a [bold white]git push[/bold white] before
    triggering the pipeline.

    Assumes that Buildkite pipeline slugs are set up per our local
    convention of [bold white]REPOSITORY-STAGE[/bold white], where
    [bold white]STAGE[/bold white] is [bold white]verify[/bold white],
    [bold white]merge[/bold white], etc.

    """
    _ensure_bk()

    repo = _repository_name()
    pipeline = f"{buildkite_organization}/{repo}-{pipeline_stage}"
    if not branch:
        branch = _current_branch()
    if not message:
        message = _message(commit)

    err_console.print(f"Git Repository: [bold white]{repo}[/bold white]")
    err_console.print(f"Git Branch: [bold white]{branch}[/bold white]")
    err_console.print(f"Git Commit: [bold white]{commit}[/bold white]")
    err_console.print(f"Buildkite Pipeline: [bold white]{pipeline}[/bold white]")
    err_console.print(f"Pipeline Message: [bold white]{message}[/bold white]")

    if push:
        _push(branch=branch, commit=commit)

    _trigger(pipeline=pipeline, branch=branch, commit=commit, message=message)


if __name__ == "__main__":
    app(prog_name="bk-run")
