from typing import cast

import toml
from cookiecutter.main import cookiecutter
from grapl_common.utils.find_grapl_root import find_grapl_root  # type: ignore
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
        self.pants_black_version_constraint = args.pants_black_version_constraint
        self.pants_isort_version_constraint = args.pants_isort_version_constraint
        self.pants_mypy_version_constraint = args.pants_mypy_version_constraint
        self.lambda_handler = args.lambda_handler

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
        self.project_path = self.python_src_path / self.project_name

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
                "pants_black_version_constraint": self.pants_black_version_constraint,
                "pants_isort_version_constraint": self.pants_isort_version_constraint,
                "pants_mypy_version_constraint": self.pants_mypy_version_constraint,
                "lambda_handler": self.lambda_handler,
            },
        )

    def attach_to_pants_toml(self) -> None:
        ...

    def precheck(self) -> None:
        # Check for the project already existing
        # Check for project already in pants.toml
        self.check_pants()
        self.check_project_exists()

    def check_project_exists(self) -> None:
        if self.project_path.exists():
            raise ValueError(f"Project already exists at {self.project_path}")

    def check_pants(self) -> None:
        pants_toml: PantsToml = cast(
            PantsToml, toml.load(self.grapl_root / "pants.toml")
        )
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
