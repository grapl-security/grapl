from __future__ import annotations

import typer
from grapl_template_generator.python_http_service.create_python_http_service import (
    create_python_http_service,
)
from grapl_template_generator.python_http_service.create_python_http_service_args import (
    CreatePythonHttpServiceArgs,
)
from grapl_template_generator.rust_grpc_service.create_rust_grpc_service import (
    create_rust_grpc_service,
)
from grapl_template_generator.rust_grpc_service.create_rust_grpc_service_args import (
    CreateRustGrpcServiceArgs,
)

"""
Generators for:
* Asynchronous/Synchronous services in Rust and Python
"""

app = typer.Typer()


@app.command(name="py-http")
def py_http(
    project_name: str,
    pants_version: str = typer.Argument("2.4.0"),
    pants_python_interpreter_constraints: str = typer.Argument("CPython==3.7.*"),
    pants_black_version_constraint: str = typer.Argument("black==20.8b1"),
    pants_isort_version_constraint: str = typer.Argument("isort==5.6.4"),
    pants_mypy_version_constraint: str = typer.Argument("mypy==0.8"),
    lambda_handler: str = "lambda_handler",
) -> None:
    args = CreatePythonHttpServiceArgs(
        project_name=project_name,
        pants_version=pants_version,
        pants_python_interpreter_constraints=pants_python_interpreter_constraints,
        pants_black_version_constraint=pants_black_version_constraint,
        pants_isort_version_constraint=pants_isort_version_constraint,
        pants_mypy_version_constraint=pants_mypy_version_constraint,
        lambda_handler=lambda_handler,
    )
    create_python_http_service(args)


@app.command(name="rust-grpc")
def rust_grpc(
    package_name: str,
    cargo_version: str = typer.Argument("1.52.0"),
    rustc_channel: str = typer.Argument("stable"),
) -> None:
    args = CreateRustGrpcServiceArgs(
        package_name=package_name,
        cargo_version=cargo_version,
        rustc_channel=rustc_channel,
    )
    create_rust_grpc_service(args)


if __name__ == "__main__":
    app()
