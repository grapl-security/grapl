from pathlib import Path
from typing import Tuple, cast

import toml
import typer
from cookiecutter.main import cookiecutter
from grapl_common.utils.find_grapl_root import find_grapl_root
from grapl_template_generator.pants_toml_type import PantsToml
from grapl_template_generator.python_http_service.create_python_http_service_args import (
    CreatePythonHttpServiceArgs,
)


class PythonHttpServiceTemplateExecutor(object):
    def __init__(self, args: CreatePythonHttpServiceArgs) -> None:
        self.project_name = args.project_name
        self.project_slug = args.project_name.lower().replace(" ", "-")
        self.pkg_name = args.project_name.lower().replace(" ", "_")
        self.pants_version = args.pants_version
        self.pants_python_interpreter_constraints = (
            args.pants_python_interpreter_constraints
        )
        self.lambda_handler = args.lambda_handler

        # TODO: In the future, it might prove more robust to package these
        # templates as a resources() goal, as opposed to just reading from src/
        grapl_root = find_grapl_root()
        assert grapl_root, "Expected to find Grapl root"

        self.grapl_root = grapl_root
        self.python_src_path = self.grapl_root / "src" / "python"
        self.template_path = (
            self.python_src_path
            / "grapl-template-generator"
            / "grapl_template_generator"
            / "grapl-templates"
            / "grapl-python-service-template"
        )
        self.project_path = self.python_src_path / self.project_slug

    def execute_template(self) -> None:
        cookiecutter(
            str(self.template_path),
            no_input=True,
            output_dir=self.python_src_path,
            extra_context={
                "project_name": self.project_name,
                "project_slug": self.project_slug,
                "pkg_name": self.pkg_name,
                "pants_version": self.pants_version,
                "pants_python_interpreter_constraints": self.pants_python_interpreter_constraints,
                "lambda_handler": self.lambda_handler,
            },
        )

    def attach_to_pants_toml(self) -> None:
        # Theoretically, we could automate this step. Unfortunately, the python
        # toml encoder doesn't want to play nicely with our existing comments.

        new_root_pattern = f"/{self.project_path.relative_to(self.grapl_root)}"
        assert new_root_pattern.startswith(
            "/src/python"
        ), f"Unexpected root pattern {new_root_pattern}"

        typer.echo(
            f"NOTE: Please add {new_root_pattern} to pants.toml's source[root_patterns]"
        )

    def precheck(self) -> None:
        # Check for the project already existing
        # Check for project already in pants.toml
        self.check_pants()
        self.check_project_exists()

    def check_project_exists(self) -> None:
        if self.project_path.exists():
            raise ValueError(f"Project already exists at {self.project_path}")

    def get_pants_toml(self) -> Tuple[Path, PantsToml]:
        path = self.grapl_root / "pants.toml"
        pants_toml = cast(PantsToml, toml.load(path))
        return path, pants_toml

    def check_pants(self) -> None:
        _, pants_toml = self.get_pants_toml()
        root_patterns = pants_toml["source"]["root_patterns"]

        for pat in root_patterns:
            if not isinstance(pat, str):
                raise TypeError(f"Invalid root_patterns {pat}")
            if pat.endswith(f"/{self.pkg_name}"):
                raise ValueError(f"Package already exists in pants.toml: {pat}")
        return None


def create_python_http_service(args: CreatePythonHttpServiceArgs) -> None:
    executor = PythonHttpServiceTemplateExecutor(args)
    executor.precheck()
    executor.execute_template()
    executor.attach_to_pants_toml()
