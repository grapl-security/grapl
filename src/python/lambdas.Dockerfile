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


# e2e-tests
################################################################################
FROM grapl-python-runner-base AS e2e-tests

COPY --chown=grapl ./dist/e2e-test-runner-lambda.zip lambda.zip

# in Docker-Compose world, we mounted `etc`; now we just copy it in.
RUN mkdir -p /home/grapl/etc
COPY --chown=grapl ./etc/local_grapl etc/local_grapl
RUN mkdir -p /home/grapl/etc/sample_data
COPY --chown=grapl ./etc/sample_data/eventlog.xml etc/sample_data/eventlog.xml

RUN unzip lambda.zip