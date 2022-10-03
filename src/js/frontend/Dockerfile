# syntax=docker/dockerfile:1.4
ARG TYPESCRIPT_VERSION

# grapl-javascript-base
################################################################################
FROM typescript:${TYPESCRIPT}-bullseye-slim AS grapl-typescript-base
SHELL ["/bin/bash", "-c"]
RUN apt-get update \
    && apt-get -y install --no-install-recommends \
        bash=5.1-2+deb11u1 \
        libstdc++6=10.2.1-6 \
        curl=7.74.0-1.3+deb11u3 \
    && rm -rf /var/lib/apt/lists/* \
    && adduser \
        --disabled-password \
        --gecos '' \
        --home /home/grapl \
        --shell /bin/bash \
        grapl
# Uncomment to get the versions for pins
#RUN dpkg-query -l && exit 5

USER grapl
WORKDIR /home/grapl
ENTRYPOINT ["/bin/bash", "-o", "errexit", "-o", "nounset", "-c"]

# Provisioner
################################################################################
FROM grapl-typescript-base AS provisioner-deploy
# Named context support https://github.com/hadolint/hadolint/issues/830
# hadolint ignore=DL3022
COPY --from=dist-ctx provisioner.pex .
CMD [":"]

# integration-tests
################################################################################
FROM grapl-typescript-base AS integration-tests
SHELL ["/bin/bash", "-o", "pipefail", "-c"]
USER root
# We install typescript so that we have a ts version supported by pants (No 3.10 support yet)
RUN apt-get update \
    && apt-get -y install --no-install-recommends \
        build-essential=12.9 \
        curl=7.74.0-1.3+deb11u3 \
        unzip=6.0-26+deb11u1 \
        typescript-dev=^4.4.2 \
    && rm -rf /var/lib/apt/lists/* \
    && adduser \
        --disabled-password \
        --gecos '' \
        --home /home/grapl \
        --shell /bin/bash \
        --uid 2000 \
        pants_ci
# Uncomment to get the versions for pins
#RUN dpkg-query -l && exit 5

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
CMD ["./test/run/typescript-integration-tests-from-docker-container.sh"]
