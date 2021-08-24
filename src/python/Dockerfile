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
COPY ./dist/analyzer-executor.pex /home/grapl/analyzer-executor.pex
CMD ["./analyzer-executor.pex"]

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
USER grapl
