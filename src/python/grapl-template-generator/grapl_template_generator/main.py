from __future__ import annotations

import typer

"""
Generators for:
* Asynchronous/Synchronous services in Rust and Python
"""

app = typer.Typer()

# @app.command()
# def create_python_http_service(args: CreatePythonHttpServiceArgs):
#     return None


if __name__ == "__main__":
    app()

# if __name__ == "__main__":
#     grapl_root = find_grapl_root()
#     if not grapl_root:
#         ...
#
#     typer.run(main)
#
#
# def generate_python_async():
#     cookiecutter('grapl-python-service-template/')
