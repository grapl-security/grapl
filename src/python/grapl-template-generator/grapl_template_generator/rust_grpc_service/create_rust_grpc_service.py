from pathlib import Path
from typing import Tuple, cast

import toml
import typer
from cookiecutter.main import cookiecutter
from grapl_common.utils.find_grapl_root import find_grapl_root
from grapl_template_generator.rust_grpc_service.create_rust_grpc_service_args import (
    CreateRustGrpcServiceArgs,
)
from grapl_template_generator.workspace_toml_type import WorkspaceToml


def camel_case_ify(input: str) -> str:
    output = ""
    for word in input.split(" "):
        output += word.capitalize()
    return output


class RustGrpcServiceTemplateExecutor(object):
    def __init__(self, args: CreateRustGrpcServiceArgs) -> None:
        self.crate_name = camel_case_ify(args.crate_name.replace("-", " "))
        self.project_slug = args.crate_name
        self.snake_project_name = self.project_slug.lower().replace("-", "_")
        self.snake_project_name_caps = (
            self.snake_project_name.upper()
        )  # for env variables

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
        self.project_path = self.rust_src_path / self.project_slug
        # We also have to manually move the generated protos to a specific directory.
        self.proto_destination = (
            self.grapl_root / "src/proto/graplinc/grapl/api" / self.snake_project_name
        )

    def precheck(self) -> None:
        pass

    def execute_template(self) -> None:
        cookiecutter(
            str(self.template_path),
            no_input=True,
            output_dir=self.rust_src_path,
            extra_context={
                "crate_name": self.crate_name,
                "project_slug": self.project_slug,
                "snake_project_name": self.snake_project_name,
                "snake_project_name_caps": self.snake_project_name_caps,
            },
        )
        self.move_protos_to_global_proto_dir()

    def move_protos_to_global_proto_dir(self) -> None:
        self.proto_destination.mkdir(exist_ok=True)
        proto_filenames = [
            f"{self.snake_project_name}.proto",
            f"{self.snake_project_name}_health.proto",
        ]
        for proto_filename in proto_filenames:
            proto_file = Path(self.project_path / "proto" / proto_filename).resolve(
                strict=True
            )
            proto_file.rename(self.proto_destination / proto_filename)

    def get_toml_for_workspace(self) -> Tuple[Path, WorkspaceToml]:
        workspace_path = self.rust_src_path / "Cargo.toml"
        workspace_toml = cast(WorkspaceToml, toml.load(workspace_path))
        return workspace_path, workspace_toml

    def attach_to_workspace(self) -> None:
        # Theoretically, we could automate this step. Unfortunately, the python
        # toml encoder/decoder doesn't want to play nicely with our comments.
        # https://github.com/uiri/toml/issues/371
        new_workspace_member = f"./{self.project_slug}"
        typer.echo(
            f"NOTE: Please add {new_workspace_member} to cargo.toml's [workspace][members]"
        )

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
