# syntax=docker/dockerfile:1.4

# grapl-python-base
################################################################################
FROM python:3.8-slim-bullseye AS grapl-python-base
SHELL ["/bin/bash", "-c"]
RUN apt-get update \
    && apt-get -y install --no-install-recommends \
        bash=5.1-2+b3 \
        libstdc++6=10.2.1-6 \
        curl=7.74.0-1.3+deb11u1 \
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

# engagement-creator
################################################################################
FROM grapl-python-base AS engagement-creator-deploy
# Named context support https://github.com/hadolint/hadolint/issues/830
# hadolint ignore=DL3022
COPY --from=dist-ctx engagement-creator.pex .
CMD ["./engagement-creator.pex"]

# Provisioner
################################################################################
FROM grapl-python-base AS provisioner-deploy
# Named context support https://github.com/hadolint/hadolint/issues/830
# hadolint ignore=DL3022
COPY --from=dist-ctx provisioner.pex .
CMD [":"]

# integration-tests
################################################################################
FROM grapl-python-base AS integration-tests
USER root
RUN apt-get update \
    && apt-get -y install --no-install-recommends \
        build-essential=12.9 \
        curl=7.74.0-1.3+deb11u1 \
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
CMD ["./test/run/py-integration-tests-from-docker-container.sh"]
