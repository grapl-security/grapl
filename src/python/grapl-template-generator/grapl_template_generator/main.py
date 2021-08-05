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
    lambda_handler: str = "lambda_handler",
) -> None:
    args = CreatePythonHttpServiceArgs(
        project_name=project_name,
        pants_version=pants_version,
        pants_python_interpreter_constraints=pants_python_interpreter_constraints,
        lambda_handler=lambda_handler,
    )
    create_python_http_service(args)
    typer.echo(f"Created a Python HTTP service named {project_name}")


@app.command(name="rust-grpc")
def rust_grpc(
    package_name: str,
) -> None:
    args = CreateRustGrpcServiceArgs(
        package_name=package_name,
    )
    create_rust_grpc_service(args)
    typer.echo(f"Created a Rust GRPC service named {package_name}")


if __name__ == "__main__":
    app()
