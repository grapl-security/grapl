# This defines a base image for running Python applications in a container
# meant to simulate AWS Lambda locally.
#
# This assumes the applications have previously been built and is only for
# creating the deployment containers.
#
# If we are to continue using AWS Lambda we should consider switching from
# deploying zips to deploying container images built from
# public.ecr.aws/lambda/python:3.7.  See https://gallery.ecr.aws/lambda/python
FROM python:3.7-slim-buster AS grapl-python-lambda-runner-base

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
RUN mkdir -p /home/grapl/.aws
RUN echo "[default]\naws_access_key_id=fake\naws_secret_access_key=fake" > /home/grapl/.aws/credentials
ENV PATH=/home/grapl/bin:$PATH

# Copy in graplctl
COPY --chown=grapl ./bin/graplctl /home/grapl/bin/graplctl


# e2e-tests
################################################################################
FROM grapl-python-lambda-runner-base AS e2e-tests

COPY --chown=grapl ./dist/e2e-test-runner-lambda.zip lambda.zip

RUN unzip lambda.zip


# provisioner
################################################################################
FROM grapl-python-lambda-runner-base AS provisioner

COPY --chown=grapl ./dist/provisioner-lambda.zip lambda.zip

RUN unzip lambda.zip
