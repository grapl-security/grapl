from pathlib import Path
from typing import Tuple, cast

import toml
from cookiecutter.main import cookiecutter
from grapl_common.utils.find_grapl_root import find_grapl_root
from grapl_template_generator.rust_grpc_service.create_rust_grpc_service_args import (
    CreateRustGrpcServiceArgs,
)
from grapl_template_generator.workspace_toml_type import WorkspaceToml


def capitalize_words(input: str) -> str:
    output = ""
    for word in input.split(" "):
        output += word.capitalize()
    return output


class RustGrpcServiceTemplateExecutor(object):
    def __init__(self, args: CreateRustGrpcServiceArgs) -> None:
        self.project_name = capitalize_words(args.package_name.replace("-", " "))
        self.project_slug = args.package_name
        self.service_name = self.project_name.replace(" ", "")
        self.snake_project_name = self.project_slug.lower().replace("-", "_")
        self.cargo_version = args.cargo_version
        self.rustc_channel = args.rustc_channel

        # TODO: In the future, it might prove more robust to package these
        # templates as a resources() goal, as opposed to just reading from src/
        grapl_root = find_grapl_root()
        assert grapl_root, "Expected to find Grapl root"

        self.grapl_root = grapl_root
        self.rust_src_path = self.grapl_root / "src" / "rust"
        self.python_src_path = self.grapl_root / "src" / "python"
        self.template_path = (
            self.python_src_path
            / "grapl-template-generator"
            / "grapl_template_generator"
            / "grapl-templates"
            / "rust-grpc-service"
        )
        self.project_path = self.rust_src_path / self.project_name

    def precheck(self) -> None:
        ...

    def execute_template(self) -> None:
        cookiecutter(
            str(self.template_path),
            no_input=True,
            output_dir=self.rust_src_path,
            extra_context={
                "project_name": self.project_name,
                "project_slug": self.project_slug,
                "service_name": self.service_name,
                "snake_project_name": self.snake_project_name,
                "cargo_version": self.cargo_version,
                "rustc_channel": self.rustc_channel,
            },
        )

    def get_toml_for_workspace(self) -> Tuple[Path, WorkspaceToml]:
        workspace_path = self.rust_src_path / "Cargo.toml"
        workspace_toml = cast(WorkspaceToml, toml.load(workspace_path))
        return workspace_path, workspace_toml

    def attach_to_workspace(self) -> None:
        workspace_path, workspace_toml = self.get_toml_for_workspace()
        workspace_toml["workspace"]["members"].append(f"./{self.project_slug}")
        workspace_toml["workspace"]["members"].sort()
        with open(workspace_path, "w") as f:
            toml_str = toml.dumps(workspace_toml)
            f.write(toml_str)

    def check_workspace(self) -> None:
        _, workspace_toml = self.get_toml_for_workspace()
        for member in workspace_toml["workspace"]["members"]:
            if member.endswith(self.project_slug):
                raise ValueError(f"Member already exists in workspace {member}")


def create_rust_grpc_service(args: CreateRustGrpcServiceArgs) -> None:
    executor = RustGrpcServiceTemplateExecutor(args)
    executor.precheck()
    executor.execute_template()
    executor.attach_to_workspace()
