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
    crate_name: str,
    hax_update_cargo_toml: bool = typer.Option(
        False,
        help="Update the Cargo.toml. You *may* lose some comments. Not recommended for anything but testing.",
    ),
) -> None:
    args = CreateRustGrpcServiceArgs(
        crate_name=crate_name, update_cargo_toml=hax_update_cargo_toml
    )
    create_rust_grpc_service(args)
    typer.echo(f"Created a Rust GRPC service named {crate_name}")


if __name__ == "__main__":
    app()
