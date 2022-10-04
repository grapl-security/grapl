# syntax=docker/dockerfile:1.4

# grapl-typescript-base
################################################################################
FROM node:18-bullseye-slim AS grapl-typescript-base
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

# integration-tests
################################################################################
FROM grapl-typescript-base AS integration-tests
SHELL ["/bin/bash", "-o", "pipefail", "-c"]
USER root
RUN apt-get update \
    && apt-get -y install --no-install-recommends \
        build-essential=12.9 \
        curl=7.74.0-1.3+deb11u3 \
        unzip=6.0-26+deb11u1 \
        typescript-dev=^4.4.2 \
    && rm -rf /var/lib/apt/lists/*
# Uncomment to get the versions for pins
#RUN dpkg-query -l && exit 5

# /mnt/grapl-root is mounted by `integration-tests.nomad` at runtime.
WORKDIR /mnt/grapl-root
CMD ["./test/run/typescript-integration-tests-from-docker-container.sh"]
