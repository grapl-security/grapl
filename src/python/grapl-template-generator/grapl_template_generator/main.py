from __future__ import annotations

import typer
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


@app.command(name="rust-grpc", help="Create a Rust gRPC project.")
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
