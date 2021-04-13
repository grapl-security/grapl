ARG TAG=latest

# python build
################################################################################
FROM grapl/grapl-python-base:$TAG AS grapl-python-build
RUN pip install wheel grpcio chalice hypothesis pytest pytest-xdist

# test deps
################################################################################

FROM grapl-python-build AS python-test-deps
COPY --chown=grapl python/python_test_deps python_test_deps
RUN python_test_deps/download_requirements.sh

# graplinc-grapl-api
################################################################################

# build
FROM grapl-python-build AS graplinc-grapl-api-build
COPY --chown=grapl proto proto
COPY --chown=grapl python/graplinc-grapl-api python/graplinc-grapl-api
SHELL ["/bin/bash", "-eu", "-o", "pipefail", "-c"]
RUN cd python/graplinc-grapl-api && \
    python setup.py sdist bdist_wheel && \
    pip install dist/*.whl

# test
FROM graplinc-grapl-api-build AS graplinc-grapl-api-test
COPY --chown=grapl --from=python-test-deps /home/grapl/python_test_deps python_test_deps
RUN python_test_deps/install_requirements.sh

# grapl-common
################################################################################

# build
FROM grapl-python-build AS grapl-common-build
COPY --chown=grapl python/grapl-common grapl-common
RUN cd grapl-common && \
    pip install . && \
    python setup.py sdist bdist_wheel

# test
FROM grapl-common-build AS grapl-common-test
COPY --chown=grapl --from=python-test-deps /home/grapl/python_test_deps python_test_deps
RUN python_test_deps/install_requirements.sh

# grapl_analyzerlib
################################################################################

# build
FROM grapl-python-build AS grapl-analyzerlib-build
COPY --chown=grapl python/grapl_analyzerlib grapl_analyzerlib
COPY --chown=grapl --from=graplinc-grapl-api-build /home/grapl/venv venv
COPY --chown=grapl --from=grapl-common-build /home/grapl/grapl-common grapl-common
RUN cd grapl-common && \
    pip install .
RUN cd grapl_analyzerlib && \
    pip install . && \
    python setup.py sdist bdist_wheel

# test
FROM grapl-analyzerlib-build AS grapl-analyzerlib-test
COPY --chown=grapl --from=python-test-deps /home/grapl/python_test_deps python_test_deps
RUN python_test_deps/install_requirements.sh


# analyzer-executor
################################################################################

# build
FROM grapl-python-build AS analyzer-executor-build
COPY --chown=grapl python/analyzer_executor analyzer_executor
COPY --chown=grapl --from=grapl-analyzerlib-build /home/grapl/venv venv
RUN cd analyzer_executor && \
    pip install .

# deploy
FROM grapl/grapl-python-base:$TAG AS analyzer-executor
COPY --chown=grapl --from=analyzer-executor-build /home/grapl/venv venv
COPY --chown=grapl --from=analyzer-executor-build /home/grapl/analyzer_executor analyzer_executor
CMD python3 analyzer_executor/src/run.py

# test
FROM analyzer-executor-build AS analyzer-executor-test
ENV IS_LOCAL=True
ENV IS_RETRY=False
COPY --chown=grapl --from=python-test-deps /home/grapl/python_test_deps python_test_deps
RUN python_test_deps/install_requirements.sh

# engagement-creator
################################################################################

# build
FROM grapl-python-build AS engagement-creator-build
COPY --chown=grapl python/engagement-creator engagement-creator
COPY --chown=grapl --from=grapl-analyzerlib-build /home/grapl/venv venv
RUN cd engagement-creator && \
    pip install .

# deploy
FROM grapl/grapl-python-base:$TAG AS engagement-creator
COPY --chown=grapl --from=engagement-creator-build /home/grapl/venv venv
COPY --chown=grapl --from=engagement-creator-build /home/grapl/engagement-creator engagement-creator
CMD python3 /engagement-creator/src/engagement_creator.py

# test
FROM engagement-creator-build AS engagement-creator-test
COPY --chown=grapl --from=python-test-deps /home/grapl/python_test_deps python_test_deps
RUN python_test_deps/install_requirements.sh
RUN cd engagement-creator && \
    pip install .[typecheck]

# engagement-edge
################################################################################

# build
FROM grapl-python-build AS engagement-edge-build
COPY --chown=grapl python/engagement_edge engagement_edge
COPY --chown=grapl --from=grapl-analyzerlib-build /home/grapl/venv venv
RUN cd engagement_edge && \
    pip install .

# zip
FROM engagement-edge-build AS engagement-edge-zip
RUN LAMBDA_DIR=$(pwd); \
    cd ~/venv/lib/python3.7/site-packages/ && \
    zip -q9r -dg "${LAMBDA_DIR}/lambda.zip" ./ && \
    cd ~/engagement_edge/src && \
    find . -type f -name "*.py" -exec zip -g "${LAMBDA_DIR}/lambda.zip" "{}" \;

# deploy
FROM grapl/grapl-python-base:$TAG AS engagement-edge
COPY --chown=grapl --from=engagement-edge-build /home/grapl/venv venv
RUN chalice new-project app/
COPY --chown=grapl --from=engagement-edge-build /home/grapl/engagement_edge/src/engagement_edge.py app/app.py
CMD cd app && \
    chalice local --no-autoreload --host=0.0.0.0 --port=8900

# test
FROM engagement-edge-build AS engagement-edge-test
ENV IS_LOCAL=True
ENV DEPLOYMENT_NAME=local-grapl
ENV UX_BUCKET_URL="ux_bucket_url"
COPY --chown=grapl --from=python-test-deps /home/grapl/python_test_deps python_test_deps
RUN python_test_deps/install_requirements.sh
COPY --chown=grapl python/grapl-tests-common grapl-tests-common
RUN cd grapl-tests-common && \
    pip install .

# grapl-ux-router
################################################################################

# build
FROM grapl-python-build AS grapl-ux-router-build
COPY --chown=grapl python/grapl-ux-router grapl-ux-router
COPY --chown=grapl --from=grapl-common-build /home/grapl/grapl-common grapl-common
RUN cd grapl-common && \
    pip install .
RUN cd grapl-ux-router && \
    pip install .

# zip
FROM grapl-ux-router-build AS ux-router-zip
RUN ORIG_DIR=$(pwd); \
    cd ~/venv/lib/python3.7/site-packages/ && \
    zip -q9r -dg "${ORIG_DIR}/lambda.zip" ./ && \
    cd ~/grapl-ux-router/src && \
    find . -type f -name "*.py" -exec zip -g "${ORIG_DIR}/lambda.zip" "{}" \;

# deploy
FROM grapl/grapl-python-base:$TAG AS grapl-ux-router
COPY --chown=grapl --from=grapl-ux-router-build /home/grapl/venv venv
RUN chalice new-project app/
COPY --chown=grapl --from=grapl-ux-router-build /home/grapl/grapl-ux-router/src/grapl_ux_router.py app/app.py
CMD cd app && \
    chalice local --no-autoreload --host=0.0.0.0 --port=8901

# dgraph-ttl
################################################################################

# build
FROM grapl-python-build AS dgraph-ttl-build
COPY --chown=grapl python/grapl-dgraph-ttl dgraph-ttl
COPY --chown=grapl --from=grapl-analyzerlib-build /home/grapl/venv venv
RUN pip install -r dgraph-ttl/requirements.txt

# zip
FROM dgraph-ttl-build AS dgraph-ttl-zip
RUN LAMBDA_DIR=$(pwd); \
    cd ~/venv/lib/python3.7/site-packages && \
    zip -q9r -dg "${LAMBDA_DIR}/lambda.zip" ./ && \
    cd ~/dgraph-ttl && \
    zip -g "${LAMBDA_DIR}/lambda.zip" app.py

# deploy
FROM grapl/grapl-python-base:$TAG AS dgraph-ttl
COPY --chown=grapl --from=dgraph-ttl-build /home/grapl/venv venv
COPY --chown=grapl --from=dgraph-ttl-build /home/grapl/dgraph-ttl app
CMD cd app && \
    chalice local --no-autoreload --host=0.0.0.0 --port=8124

# test
FROM dgraph-ttl-build AS dgraph-ttl-test
COPY --chown=grapl --from=python-test-deps /home/grapl/python_test_deps python_test_deps
RUN python_test_deps/install_requirements.sh


# model-plugin-deployer
################################################################################

# build
FROM grapl-python-build AS model-plugin-deployer-build
COPY --chown=grapl python/grapl-model-plugin-deployer model-plugin-deployer
COPY --chown=grapl --from=grapl-analyzerlib-build /home/grapl/venv venv
RUN cd model-plugin-deployer && \
    pip install .

# zip
FROM model-plugin-deployer-build AS model-plugin-deployer-zip
RUN LAMBDA_DIR=$(pwd); \
    cd ~/venv/lib/python3.7/site-packages && \
    zip -q9r -dg "${LAMBDA_DIR}/lambda.zip" ./ && \
    cd ~/model-plugin-deployer/src && \
    find . -type f -name "*.py" -exec zip -g "${LAMBDA_DIR}/lambda.zip" "{}" \;

# deploy
FROM grapl/grapl-python-base:$TAG AS model-plugin-deployer
COPY --chown=grapl --from=model-plugin-deployer-build /home/grapl/venv venv
RUN chalice new-project app/
COPY --chown=grapl --from=model-plugin-deployer-build /home/grapl/model-plugin-deployer/src/grapl_model_plugin_deployer.py app/app.py
CMD cd app && \
    chalice local --no-autoreload --host=0.0.0.0 --port=8123

# test
FROM model-plugin-deployer-build AS model-plugin-deployer-test
COPY --chown=grapl --from=python-test-deps /home/grapl/python_test_deps python_test_deps
RUN python_test_deps/install_requirements.sh

# Notebook
################################################################################

# build
FROM grapl/grapl-python-base:$TAG AS grapl-notebook
EXPOSE 8888
COPY --chown=grapl --from=grapl-analyzerlib-build /home/grapl/venv venv
RUN pip install jupyter
RUN mkdir -p grapl-notebook/model_plugins
COPY --chown=grapl python/grapl-notebook/jupyter_notebook_config.py /home/grapl/.jupyter/
COPY --chown=grapl python/grapl-notebook/Demo_Engagement.ipynb grapl-notebook/
CMD cd grapl-notebook && \
    jupyter notebook --ip="0.0.0.0"

# test
FROM grapl-notebook AS grapl-notebook-test
RUN pip install nbqa mypy boto3-stubs[essential]

#
# Local provision
################################################################################

# build
FROM grapl/grapl-python-base:$TAG AS grapl-provision
COPY --chown=grapl python/grapl_provision grapl_local_provision
COPY --chown=grapl --from=grapl-analyzerlib-build /home/grapl/venv venv

# grapl-tests-common-build
################################################################################
FROM grapl-python-build AS grapl-tests-common-build
COPY --chown=grapl --from=grapl-analyzerlib-build /home/grapl/venv venv
COPY --from=python-test-deps /home/grapl/python_test_deps python_test_deps
RUN python_test_deps/install_requirements.sh
COPY --chown=grapl python/grapl-tests-common grapl-tests-common
RUN cd grapl-tests-common && \
    pip install . && \
    python setup.py sdist bdist_wheel

# GraphQL Endpoint tests
################################################################################
FROM grapl-tests-common-build AS graphql-endpoint-tests
COPY --chown=grapl python/graphql_endpoint_tests graphql_endpoint_tests

# E2E tests
################################################################################
FROM grapl-tests-common-build AS grapl-e2e-tests-build
COPY --chown=grapl python/grapl_e2e_tests grapl_e2e_tests
# Allow user `grapl` to write to the mount point
USER root
RUN mkdir /mnt/dynamodb_dump && chown grapl /mnt/dynamodb_dump
USER grapl
RUN pip install zstd
