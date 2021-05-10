import os
from typing import MutableMapping, Any, cast

from cookiecutter.main import cookiecutter

from grapl_common.utils.find_grapl_root import find_grapl_root  # type: ignore
from grapl_template_generator.pants_toml_type import PantsToml
from grapl_template_generator.python_http_service.create_python_http_service_args import CreatePythonHttpServiceArgs

from os.path import join as pathjoin

import toml

# grapl-templates/grapl-python-service-template/cookiecutter.json
# {
#     "project_name": "My New Project",
#     "project_slug": "{{ cookiecutter.project_name|lower|replace(' ', '-') }}",
#     "pkg_name": "{{ cookiecutter.project_slug|replace('-', '_') }}",
#     "pants_version": "2.5.0",
#     "pants_python_interpreter_constraints": "CPython==3.7.*",
#     "pants_black_version_constraint": "black==20.8b1",
#     "pants_isort_version_constraint": "isort==5.6.4",
#     "pants_mypy_version_constraint": "mypy==0.800",
#     "lambda_handler": "lambda_handler"
# }


class PythonHttpServiceTemplateExecutor(object):
    def __init__(self, args: CreatePythonHttpServiceArgs) -> None:
        self.project_name = args.project_name
        self.project_slug = args.project_name.lower().replace(' ', '-')
        self.pkg_name = args.project_name.lower().replace(' ', '_')
        self.pants_version = args.pants_version
        self.pants_python_interpreter_constraints = args.pants_python_interpreter_constraints
        self.pants_black_version_constraint = args.pants_black_version_constraint
        self.pants_isort_version_constraint = args.pants_isort_version_constraint
        self.pants_mypy_version_constraint = args.pants_mypy_version_constraint
        self.lambda_handler = args.lambda_handler

        self.grapl_root = find_grapl_root()  # type: str
        self.python_src_path = pathjoin(self.grapl_root, "src/python/")
        self.template_path = pathjoin(
            self.python_src_path,
            "grapl-template-generator/grapl_template_generator/grapl-templates/grapl-python-service-template/",
        )
        self.project_path = pathjoin(self.python_src_path, self.project_name)

    def execute_template(self) -> None:
        cookiecutter(
            self.template_path,
            no_input=True,
            output_dir=self.python_src_path,
            extra_context={
                'project_name': self.project_name,
                'project_slug': self.project_slug,
                'pkg_name': self.pkg_name,
                'pants_version': self.pants_version,
                'pants_python_interpreter_constraints': self.pants_python_interpreter_constraints,
                'pants_black_version_constraint': self.pants_black_version_constraint,
                'pants_isort_version_constraint': self.pants_isort_version_constraint,
                'pants_mypy_version_constraint': self.pants_mypy_version_constraint,
                'lambda_handler': self.lambda_handler,
            }
        )

    def attach_to_pants_toml(self) -> None:
        ...

    def precheck(self) -> None:
        # Check for the project already existing
        # Check for project already in pants.toml
        self.check_pants()
        self.check_project_exists()

    def check_project_exists(self) -> None:
        if os.path.exists(self.project_path):
            raise ValueError("Project already exists")

    def check_pants(self) -> None:
        pants_toml: PantsToml = cast(PantsToml, toml.load(pathjoin(self.grapl_root, "pants.toml")))
        root_patterns = pants_toml['source']['root_patterns']

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
