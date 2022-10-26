# This is the Docker image that Grapl plugins will run inside until we swap
# over to Firecracker. 
# It needs to be able to run Rust binaries (nothing special) and 
# python PEXs (needs py3.10).

ARG PYTHON_VERSION

FROM python:${PYTHON_VERSION}-slim-bullseye AS docker-plugin-runtime