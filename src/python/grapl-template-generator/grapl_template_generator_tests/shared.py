from typing import Sequence

from grapl_template_generator import main
from typer.testing import CliRunner, Result


def invoke_main(args: Sequence[str]) -> Result:
    return CliRunner().invoke(main.app, [*args], catch_exceptions=False)
