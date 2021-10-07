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
RUN mkdir -p /tmp/pants && chmod -R 777 /tmp/pants
USER grapl
RUN mkdir -p /tmp/pants/named_caches && chmod -R 777 /tmp/pants/named_caches
RUN mkdir -p /tmp/pants/lmdb_store && chmod -R 777 /tmp/pants/lmdb_store
RUN mkdir -p /tmp/pants/setup && chmod -R 777 /tmp/pants/setup
ENV PANTS_NAMED_CACHES_DIR=/tmp/pants/named_caches
ENV PANTS_LOCAL_STORE_DIR=/tmp/pants/lmdb_store
ENV PANTS_SETUP_CACHE=/tmp/pants/setup

# grapl-root is mounted by Nomad.
ENTRYPOINT ["/bin/bash", "-o", "errexit", "-o", "nounset", "-c", "cd /mnt/grapl-root && ./pants filter --filter-target-type='python_tests' :: | xargs ./pants --tag='-needs_work' test --pytest-args='-m integration_test'"]
