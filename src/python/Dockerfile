# grapl-python-base
################################################################################
FROM python:3.7-slim-bullseye AS grapl-python-base
SHELL ["/bin/bash", "-c"]
RUN apt-get update \
    && apt-get -y install --no-install-recommends \
        bash=5.1-2+b3 \
        libstdc++6=10.2.1-6 \
    && rm -rf /var/lib/apt/lists/* \
    && adduser \
        --disabled-password \
        --gecos '' \
        --home /home/grapl \
        --shell /bin/bash \
        grapl
USER grapl
WORKDIR /home/grapl
ENTRYPOINT ["/bin/bash", "-o", "errexit", "-o", "nounset", "-c"]

# analyzer-executor
################################################################################
FROM grapl-python-base AS analyzer-executor-deploy
COPY ./dist/analyzer-executor.pex .
CMD ["./analyzer-executor.pex"]

# engagement-creator
################################################################################
FROM grapl-python-base AS engagement-creator-deploy
COPY ./dist/engagement-creator.pex .
CMD ["./engagement-creator.pex"]

# Provisioner
################################################################################
FROM grapl-python-base AS provisioner-deploy
COPY ./dist/provisioner.pex .
CMD ["./provisioner.pex"]

# e2e tests
################################################################################
FROM grapl-python-base AS e2e-tests

# Copy in the contents of `etc` in order to upload them during e2e tests
RUN mkdir -p /home/grapl/etc
COPY --chown=grapl ./etc/local_grapl etc/local_grapl
COPY --chown=grapl ./etc/sample_data/ etc/sample_data

# Copy in `graplctl`, which does the actual uploading during e2e
RUN mkdir -p /home/grapl/bin
COPY --chown=grapl dist/src.python.graplctl.graplctl/graplctl.pex /home/grapl/bin/graplctl

COPY --chown=grapl ./dist/e2e-tests.pex .
CMD ["./e2e-tests.pex"]

ENV PATH=/home/grapl/bin:$PATH

# integration-tests
################################################################################
FROM grapl-python-base AS integration-tests
USER root
RUN apt-get update \
    && apt-get -y install --no-install-recommends \
        build-essential=12.9 \
        curl=7.74.0-1.3+deb11u1 \
        protobuf-compiler=3.12.4-1 \
        unzip=6.0-26 \
    && rm -rf /var/lib/apt/lists/* \
    && adduser \
        --disabled-password \
        --gecos '' \
        --home /home/grapl \
        --shell /bin/bash \
        --uid 2000 \
        pants_ci

# /mnt/pants-cache is mounted by `integration-tests.nomad` at runtime.
# It will copy the existing directory structure in.
RUN mkdir -p /mnt/pants-cache && chmod -R 777 /mnt/pants-cache
USER grapl
RUN mkdir -p /mnt/pants-cache/named_caches && chmod -R 777 /mnt/pants-cache/named_caches \
    && mkdir -p /mnt/pants-cache/lmdb_store && chmod -R 777 /mnt/pants-cache/lmdb_store \
    && mkdir -p /mnt/pants-cache/setup && chmod -R 777 /mnt/pants-cache/setup
ENV PANTS_NAMED_CACHES_DIR=/mnt/pants-cache/named_caches
ENV PANTS_LOCAL_STORE_DIR=/mnt/pants-cache/lmdb_store
ENV PANTS_SETUP_CACHE=/mnt/pants-cache/setup

# /mnt/grapl-root is mounted by `integration-tests.nomad` at runtime.
WORKDIR /mnt/grapl-root
CMD ["./test/run/integration-tests-from-docker-container.sh"]
