#
# base build
#

FROM python:3.7-slim-buster AS grapl-python-build

SHELL ["/bin/bash", "-c"]

RUN apt-get update && apt-get -y install --no-install-recommends \
    build-essential \
    musl-dev \
    protobuf-compiler \
    wait-for-it \
    zip \
    && rm -rf /var/lib/apt/lists/*

ENV PROTOC /usr/bin/protoc
ENV PROTOC_INCLUDE /usr/include

RUN adduser --disabled-password --gecos '' --home /home/grapl --shell /bin/bash grapl

USER grapl
WORKDIR /home/grapl

RUN python3 -mvenv venv && \
    source venv/bin/activate && \
    pip install --upgrade pip && \
    pip install wheel grpcio chalice hypothesis pytest pytest-xdist

CMD :

#
# base deploy
#

FROM python:3.7-slim-buster AS grapl-python-deploy

SHELL ["/bin/bash", "-c"]

RUN apt-get update && apt-get -y install --no-install-recommends \
    bash \
    libstdc++6 \
    wait-for-it \
    && rm -rf /var/lib/apt/lists/*

RUN adduser --disabled-password --gecos '' --home /home/grapl --shell /bin/bash grapl

USER grapl
WORKDIR /home/grapl

RUN python3 -mvenv venv && \
    source venv/bin/activate && \
    pip install --upgrade pip

CMD :

#
# test deps
#

FROM grapl-python-build AS python-test-deps

COPY --chown=grapl python/python_test_deps python_test_deps

RUN python_test_deps/download_requirements.sh

#
# graph-descriptions
#

# build
FROM grapl-python-build AS graph-descriptions-build

COPY --chown=grapl rust/graph-descriptions graph-descriptions

RUN source venv/bin/activate && \
    cd graph-descriptions && \
    pip install . && \
    python setup.py sdist bdist_wheel

# test
FROM graph-descriptions-build AS graph-descriptions-test

## Install test requirements
COPY --chown=grapl --from=python-test-deps /home/grapl/python_test_deps python_test_deps

RUN python_test_deps/install_requirements.sh

## Run unit tests
CMD source venv/bin/activate && \
    cd graph-descriptions && \
    py.test -n auto -m "not integration_test"

#
# grapl-common
#

# build
FROM grapl-python-build AS grapl-common-build

COPY --chown=grapl python/grapl-common grapl-common

RUN source venv/bin/activate && \
    cd grapl-common && \
    pip install . && \
    python setup.py sdist bdist_wheel

FROM grapl-common-build AS grapl-common-test

## Install test requirements
COPY --chown=grapl --from=python-test-deps /home/grapl/python_test_deps python_test_deps
RUN python_test_deps/install_requirements.sh

## Run unit tests
CMD source venv/bin/activate && \
    cd grapl-common/tests && \
    py.test -n auto -m "not integration_test"

#
# grapl_analyzerlib
#

# build
FROM grapl-python-build AS grapl-analyzerlib-build

COPY --chown=grapl python/grapl_analyzerlib grapl_analyzerlib
COPY --chown=grapl --from=graph-descriptions-build /home/grapl/venv venv
COPY --chown=grapl --from=grapl-common-build /home/grapl/grapl-common grapl-common

# Install requirement `grapl_common` - we could also manually COPY the `venv/site_packages`, 
# but the pip install is cleaner.
RUN source venv/bin/activate && \
    cd grapl-common && \
    pip install .

RUN source venv/bin/activate && \
    cd grapl_analyzerlib && \
    pip install . && \
    python setup.py sdist bdist_wheel

# test
FROM grapl-analyzerlib-build AS grapl-analyzerlib-test

## Install test requirements
COPY --chown=grapl --from=python-test-deps /home/grapl/python_test_deps python_test_deps
RUN python_test_deps/install_requirements.sh

## Run unit tests
CMD source venv/bin/activate && \
    cd grapl_analyzerlib && \
    py.test -n auto -m "not integration_test"

#
# analyzer-executor
#

# build
FROM grapl-python-build AS analyzer-executor-build

COPY --chown=grapl python/analyzer_executor analyzer_executor
COPY --chown=grapl --from=grapl-analyzerlib-build /home/grapl/venv venv

RUN source venv/bin/activate && \
    cd analyzer_executor && \
    pip install .

CMD ORIG_DIR=$(pwd); \
    cd ~/venv/lib/python3.7/site-packages && \
    zip -q9r -dg "${ORIG_DIR}/lambda.zip" ./ && \
    cd ~/analyzer_executor/src && \
    find . -type f -name "*.py" -exec zip -g "${ORIG_DIR}/lambda.zip" {} \;

# deploy
FROM grapl-python-deploy AS analyzer-executor-deploy

COPY --chown=grapl --from=analyzer-executor-build /home/grapl/venv venv
COPY --chown=grapl --from=analyzer-executor-build /home/grapl/analyzer_executor analyzer_executor

CMD source venv/bin/activate && \
    python3 analyzer_executor/src/analyzer-executor.py

# test
FROM analyzer-executor-build AS analyzer-executor-test

ENV IS_LOCAL=True
ENV IS_RETRY=False

## Install test requirements
COPY --chown=grapl --from=python-test-deps /home/grapl/python_test_deps python_test_deps
RUN python_test_deps/install_requirements.sh

## Run unit tests
CMD source venv/bin/activate && \
    cd analyzer_executor && \
    export PYTHONPATH="${PYTHONPATH}:$(pwd)/src" && \
    py.test -n auto -m "not integration_test"

#
# engagement-creator
#

# build
FROM grapl-python-build AS engagement-creator-build

COPY --chown=grapl python/engagement-creator engagement-creator
COPY --chown=grapl --from=grapl-analyzerlib-build /home/grapl/venv venv

RUN source venv/bin/activate && \
    cd engagement-creator && \
    pip install .

CMD ORIG_DIR=$(pwd); \
    cd ~/venv/lib/python3.7/site-packages/ && \
    zip -q9r -dg "${ORIG_DIR}/lambda.zip" ./ && \
    cd ~/engagement-creator/src && \
    find . -type f -name "*.py" -exec zip -g "${ORIG_DIR}/lambda.zip" {} \;

# deploy
FROM grapl-python-deploy AS engagement-creator-deploy

COPY --chown=grapl --from=engagement-creator-build /home/grapl/venv venv
COPY --chown=grapl --from=engagement-creator-build /home/grapl/engagement-creator engagement-creator

CMD source /venv/bin/activate && \
    python3 /engagement-creator/src/engagement-creator.py

# test
FROM engagement-creator-build AS engagement-creator-test

## Install test requirements
COPY --chown=grapl --from=python-test-deps /home/grapl/python_test_deps python_test_deps
RUN python_test_deps/install_requirements.sh

RUN source venv/bin/activate && \
    cd engagement-creator && \
    pip install .[typecheck]

## Run unit tests
CMD source venv/bin/activate && \
    cd engagement-creator && \
    py.test -n auto -m "not integration_test"

#
# engagement-edge
#

# build
FROM grapl-python-build AS engagement-edge-build

COPY --chown=grapl python/engagement_edge engagement_edge
COPY --chown=grapl --from=grapl-analyzerlib-build /home/grapl/venv venv

RUN source venv/bin/activate && \
    cd engagement_edge && \
    pip install .

CMD ORIG_DIR=$(pwd); \
    cd ~/venv/lib/python3.7/site-packages/ && \
    zip -q9r -dg "${ORIG_DIR}/lambda.zip" ./ && \
    cd ~/engagement_edge/src && \
    find . -type f -name "*.py" -exec zip -g "${ORIG_DIR}/lambda.zip" {} \;

# deploy
FROM grapl-python-deploy AS engagement-edge-deploy

COPY --chown=grapl --from=engagement-edge-build /home/grapl/venv venv

RUN source venv/bin/activate && \
    chalice new-project app/

COPY --chown=grapl --from=engagement-edge-build /home/grapl/engagement_edge/src/engagement_edge.py app/app.py

CMD source venv/bin/activate && \
    cd app && \
    chalice local --no-autoreload --host=0.0.0.0 --port=8900

# test
FROM engagement-edge-build AS engagement-edge-test

ENV IS_LOCAL=True
ENV BUCKET_PREFIX=local-grapl
ENV UX_BUCKET_URL="ux_bucket_url"

## Install test requirements
COPY --chown=grapl --from=python-test-deps /home/grapl/python_test_deps python_test_deps
RUN python_test_deps/install_requirements.sh

## Steal and install grapl-tests-common
COPY --chown=grapl python/grapl-tests-common grapl-tests-common
RUN source venv/bin/activate && \
    cd grapl-tests-common && \
    pip install .

CMD source venv/bin/activate && \
    cd engagement_edge && \
    py.test -n auto -m "not integration_test"

#
# dgraph-ttl
#

# build
FROM grapl-python-build AS dgraph-ttl-build

COPY --chown=grapl python/grapl-dgraph-ttl dgraph-ttl
COPY --chown=grapl --from=grapl-analyzerlib-build /home/grapl/venv venv

RUN source venv/bin/activate && \
    pip install -r dgraph-ttl/requirements.txt

CMD ORIG_DIR=$(pwd); \
    cd ~/venv/lib/python3.7/site-packages && \
    zip -q9r -dg "${ORIG_DIR}/lambda.zip" ./ && \
    cd ~/dgraph-ttl && \
    zip -g "${ORIG_DIR}/lambda.zip" app.py

# deploy
FROM grapl-python-deploy AS dgraph-ttl-deploy

COPY --chown=grapl --from=dgraph-ttl-build /home/grapl/venv venv
COPY --chown=grapl --from=dgraph-ttl-build /home/grapl/dgraph-ttl app

CMD source venv/bin/activate && \
    cd app && \
    chalice local --no-autoreload --host=0.0.0.0 --port=8124

# test
FROM dgraph-ttl-build AS dgraph-ttl-test

## Install test requirements
COPY --chown=grapl --from=python-test-deps /home/grapl/python_test_deps python_test_deps
RUN python_test_deps/install_requirements.sh

CMD source venv/bin/activate && \
    cd dgraph-ttl && \
    py.test -n auto -m "not integration_test"

#
# model-plugin-deployer
#

# build
FROM grapl-python-build AS model-plugin-deployer-build

COPY --chown=grapl python/grapl-model-plugin-deployer model-plugin-deployer
COPY --chown=grapl --from=grapl-analyzerlib-build /home/grapl/venv venv

RUN source venv/bin/activate && \
    cd model-plugin-deployer && \
    pip install .

CMD ORIG_DIR=$(pwd); \
    cd ~/venv/lib/python3.7/site-packages && \
    zip -q9r -dg "${ORIG_DIR}/lambda.zip" ./ && \
    cd ~/model-plugin-deployer/src && \
    find . -type f -name "*.py" -exec zip -g "${ORIG_DIR}/lambda.zip" {} \;

# deploy
FROM grapl-python-deploy AS model-plugin-deployer-deploy

COPY --chown=grapl --from=model-plugin-deployer-build /home/grapl/venv venv

RUN source venv/bin/activate && \
    chalice new-project app/

COPY --chown=grapl --from=model-plugin-deployer-build /home/grapl/model-plugin-deployer/src/grapl_model_plugin_deployer.py app/app.py

CMD source venv/bin/activate && \
    cd app && \
    chalice local --no-autoreload --host=0.0.0.0 --port=8123

# test
FROM model-plugin-deployer-build AS model-plugin-deployer-test

## Install test requirements
COPY --chown=grapl --from=python-test-deps /home/grapl/python_test_deps python_test_deps
RUN python_test_deps/install_requirements.sh

CMD source venv/bin/activate && \
    cd model-plugin-deployer && \
    py.test -n auto -m "not integration_test"

#
# swarm-lifecycle-event-handler
#

# build
FROM grapl-python-build AS swarm-lifecycle-event-handler-build

COPY --chown=grapl python/swarm-lifecycle-event-handler swarm-lifecycle-event-handler

RUN source venv/bin/activate && \
    pip install -r swarm-lifecycle-event-handler/requirements.txt

CMD ORIG_DIR=$(pwd); \
    cd ~/venv/lib/python3.7/site-packages && \
    zip -q9r -dg "${ORIG_DIR}/lambda.zip" ./ && \
    cd ~/swarm-lifecycle-event-handler && \
    find . -type f -name "*.py" -exec zip -g "${ORIG_DIR}/lambda.zip" {} \;

# deploy
FROM grapl-python-deploy AS swarm-lifecycle-event-handler-deploy

# placeholder for now

# test
FROM python-test-deps AS swarm-lifecycle-event-handler-test

## Install test requirements
COPY --chown=grapl --from=python-test-deps /home/grapl/python_test_deps python_test_deps
RUN python_test_deps/install_requirements.sh

CMD :

#
# Notebook
#

# build
# FROM grapl-python-build AS grapl-notebook
FROM grapl-python-deploy AS grapl-notebook

SHELL ["/bin/bash", "-o", "pipefail", "-c"]
EXPOSE 8888

COPY --chown=grapl --from=grapl-analyzerlib-build /home/grapl/venv venv

RUN source venv/bin/activate && \
    pip install jupyter

# Set up jupyter-notebook stuff
RUN mkdir -p grapl-notebook/model_plugins
COPY --chown=grapl python/grapl-notebook/jupyter_notebook_config.py /.jupyter/
COPY --chown=grapl python/grapl-notebook/Demo_Engagement.ipynb grapl-notebook/

## Run it
CMD source venv/bin/activate && \
    cd grapl-notebook && \
    jupyter notebook --ip="0.0.0.0"

# test
FROM grapl-notebook AS grapl-notebook-test

# This file will improve once #444 lands
# primarily to use pre-downloaded tools instead of downloading again

RUN source venv/bin/activate && \
    pip install nbqa mypy boto3-stubs[essential]

CMD source venv/bin/activate && \
    cd grapl-notebook && \
    nbqa mypy Demo_Engagement.ipynb && \
    cd /grapl-etc && \
    nbqa mypy "Grapl Provision.ipynb"

# While the grapl provision notebook technically has nothing to do with grapl-notebook,
# I've arbitrarily decided to make this the generalized "check the quality of ipynbs" image.

#
# Local provision
#

# build
FROM grapl-python-build AS grapl-provision

COPY --chown=grapl python/grapl_provision grapl_local_provision
COPY --chown=grapl --from=grapl-analyzerlib-build /home/grapl/venv venv