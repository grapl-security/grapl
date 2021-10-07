# This defines a base image for running Python applications in local containers
# that in run Lambda in our AWS deployments.
#
# This assumes the applications have previously been built and is only for
# creating the deployment containers.
FROM python:3.7-slim-buster AS grapl-python-runner-base

RUN --mount=type=cache,target=/var/lib/apt/lists \
    apt-get update && apt-get install -y --no-install-recommends \
        unzip

RUN adduser \
    --disabled-password \
    --gecos '' \
    --home /home/grapl \
    --shell /bin/bash \
    grapl

USER grapl
ENV USER=grapl
WORKDIR /home/grapl
RUN mkdir -p /home/grapl/bin

ENV PATH=/home/grapl/bin:$PATH

# Copy in graplctl
COPY --chown=grapl ./bin/graplctl /home/grapl/bin/graplctl