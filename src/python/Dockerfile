# grapl-python-base
################################################################################
FROM python:3.7-slim-buster AS grapl-python-base
SHELL ["/bin/bash", "-c"]
RUN apt-get update && apt-get -y install --no-install-recommends \
        bash \
        libstdc++6 \
    && rm -rf /var/lib/apt/lists/*
RUN adduser \
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
FROM grapl-python-base AS analyzer-executor
COPY ./dist/analyzer-executor.pex /home/grapl/
CMD ["./analyzer-executor.pex"]

# engagement-creator
################################################################################
FROM grapl-python-base AS engagement-creator
COPY ./dist/engagement-creator.pex /home/grapl/
CMD ["./engagement-creator.pex"]

# Provisioner
################################################################################
FROM grapl-python-base AS provisioner
COPY ./dist/provisioner.pex /home/grapl/
CMD ["./provisioner.pex"]

# e2e tests
################################################################################
FROM grapl-python-base AS e2e-tests
COPY ./dist/e2e-tests.pex /home/grapl/
CMD ["./e2e-tests.pex"]
# Copy in the contents of `etc` in order to upload them during e2e tests
RUN mkdir -p /home/grapl/etc
COPY --chown=grapl ./etc/local_grapl etc/local_grapl
RUN mkdir -p /home/grapl/etc/sample_data
COPY --chown=grapl ./etc/sample_data/eventlog.xml etc/sample_data/eventlog.xml
# Copy in `graplctl`, which does the actual uploading during e2e
RUN mkdir -p /home/grapl/bin
COPY --chown=grapl ./bin/graplctl /home/grapl/bin/graplctl
ENV PATH=/home/grapl/bin:$PATH

# grapl-notebook
################################################################################
FROM grapl-python-base AS grapl-notebook
EXPOSE 8888
RUN pip install jupyter
RUN mkdir -p grapl-notebook/model_plugins
COPY --chown=grapl src/python/grapl-notebook/jupyter_notebook_config.py /home/grapl/.jupyter/
COPY --chown=grapl src/python/grapl-notebook/Demo_Engagement.ipynb /home/grapl/grapl-notebook/
WORKDIR /home/grapl/grapl-notebook
CMD ["jupyter", "notebook", "--ip", "0.0.0.0"]

# python-integration-tests
################################################################################
FROM grapl-python-base AS python-integration-tests
USER root
RUN apt-get update && apt-get -y install --no-install-recommends \
        build-essential \
        protobuf-compiler \
        curl \
        unzip \
    && rm -rf /var/lib/apt/lists/*
RUN apt-get update && apt-get -y install --no-install-recommends curl unzip
RUN adduser \
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
RUN mkdir -p /mnt/pants-cache/named_caches && chmod -R 777 /mnt/pants-cache/named_caches
RUN mkdir -p /mnt/pants-cache/lmdb_store && chmod -R 777 /mnt/pants-cache/lmdb_store
RUN mkdir -p /mnt/pants-cache/setup && chmod -R 777 /mnt/pants-cache/setup
ENV PANTS_NAMED_CACHES_DIR=/mnt/pants-cache/named_caches
ENV PANTS_LOCAL_STORE_DIR=/mnt/pants-cache/lmdb_store
ENV PANTS_SETUP_CACHE=/mnt/pants-cache/setup

# /mnt/grapl-root is mounted by `integration-tests.nomad` at runtime.
WORKDIR /mnt/grapl-root
CMD ["./test/run/integration-tests-from-docker-container.sh"]
