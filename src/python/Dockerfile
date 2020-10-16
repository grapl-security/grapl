#
# base build
#

FROM python:3.7-slim-buster AS grapl-python-build

RUN apt-get update && apt-get -y install --no-install-recommends \
    musl-dev \
    protobuf-compiler \
    build-essential \
    zip \
    bash

ENV PROTOC /usr/bin/protoc
ENV PROTOC_INCLUDE /usr/include

ARG USERID="1000:1000"
USER $USERID
WORKDIR /home/grapl

RUN python3 -mvenv venv
RUN bash -c ' \
    source venv/bin/activate && \
    pip install --upgrade pip && \
    pip install wheel grpcio chalice hypothesis pytest pytest-xdist'

#
# base deploy
#

FROM python:3.7-slim-buster AS grapl-python-deploy
RUN apt-get install -y libstdc++6 bash

#
# grapl-python-lib
#

FROM grapl-python-build AS grapl-python-libs

ARG USERID="1000:1000"
USER $USERID
WORKDIR /home/grapl

COPY --chown=$USERID rust/graph-descriptions graph-descriptions
COPY --chown=$USERID python/grapl-common grapl-common
COPY --chown=$USERID python/grapl_analyzerlib grapl_analyzerlib

# graph-descriptions
RUN bash -c ' \
    source venv/bin/activate && \
    cd graph-descriptions && \
    ls -al && \
    id && \
    pip install . && \
    py.test -n auto -m "not integration_test" && \
    python setup.py sdist bdist_wheel'
# grapl-common
RUN bash -c ' \
    source venv/bin/activate && \
    cd grapl-common && \
    pip install . && \
    python setup.py sdist bdist_wheel'
# grapl_analyzerlib
RUN bash -c ' \
    source venv/bin/activate && \
    cd grapl_analyzerlib && \
    pip install . && \
    python setup.py sdist bdist_wheel'

#
# analyzer-deployer
#

FROM grapl-python-libs AS analyzer-deployer-build

ARG USERID="1000:1000"
USER $USERID
WORKDIR /home/grapl

COPY --chown=$USERID python/analyzer-deployer analyzer-deployer

RUN bash -c ' \
    source venv/bin/activate && \
    pip install -r analyzer-deployer/requirements.txt && \
    python -m mypy_boto3'

CMD bash -c ' \
    cd venv/lib/python3.7/site-packages && \
    zip --quiet -9r /home/grapl/lambda.zip . && \
    cd /home/grapl/analyzer-deployer/analyzer_deployer && \
    zip -g /home/grapl/lambda.zip app.py && \
    cp /home/grapl/lambda.zip /dist/analyzer-deployer.zip'

FROM grapl-python-deploy AS analyzer-deployer

WORKDIR /home/grapl

COPY --from=analyzer-deployer-build /home/grapl/venv venv
COPY --from=analyzer-deployer-build /home/grapl/analyzer-deployer app

CMD bash -c ' \
    source venv/bin/activate && \
    cd /home/grapl/app && \
    chalice local --no-autoreload --host=0.0.0.0 --port=8125'

#
# analyzer-executor
#

FROM grapl-python-libs AS analyzer-executor-build

ARG USERID="1000:1000"
USER $USERID
WORKDIR /home/grapl

COPY --chown=$USERID python/analyzer_executor analyzer_executor

RUN bash -c ' \
    source venv/bin/activate && \
    cd analyzer_executor && \
    pip install .'

CMD bash -c ' \
    cd venv/lib/python3.7/site-packages/ && \
    zip --quiet -9r /home/grapl/lambda.zip ./ && \
    cd /home/grapl/analyzer_executor/src/ && \
    zip -g /home/grapl/lambda.zip ./analyzer-executor.py && \
    cp /home/grapl/lambda.zip /dist/analyzer-executor.zip'

FROM grapl-python-deploy AS grapl-analyzer-executor

WORKDIR /home/grapl

COPY --from=analyzer-executor-build /home/grapl/venv venv
COPY --from=analyzer-executor-build /home/grapl/analyzer_executor analyzer_executor

CMD bash -c ' \
    source venv/bin/activate && \
    python3 analyzer_executor/src/analyzer-executor.py'

#
# engagement-creator
#

FROM grapl-python-libs AS engagement-creator-build

ARG USERID="1000:1000"
USER $USERID
WORKDIR /home/grapl

COPY --chown=$USERID python/engagement-creator engagement-creator

RUN bash -c ' \
    source venv/bin/activate && \
    cd engagement-creator && \
    pip install .'

CMD bash -c ' \
    zip --quiet -9r lambda.zip venv/lib/python3.7/site-packages/ && \
    zip -g lambda.zip engagement-creator/src/engagement-creator.py && \
    cp lambda.zip /dist/engagement-creator.zip'

FROM grapl-python-deploy AS grapl-engagement-creator

ARG USERID="1000:1000"
USER $USERID
WORKDIR /home/grapl

COPY --from=engagement-creator-build /home/grapl/venv venv
COPY --from=engagement-creator-build /home/grapl/engagement-creator engagement-creator

CMD bash -c ' \
    source venv/bin/activate && \
    python3 engagement-creator/src/engagement-creator.py'

#
# engagement-edge
#

FROM grapl-python-libs AS engagement-edge-build

ARG USERID="1000:1000"
USER $USERID
WORKDIR /home/grapl

COPY --chown=$USERID python/engagement_edge engagement_edge

RUN bash -c ' \
    source venv/bin/activate && \
    cd engagement_edge && \
    pip install .'

CMD bash -c ' \
    cd venv/lib/python3.7/site-packages/ && \
    zip --quiet -9r /home/grapl/lambda.zip ./ && \
    cd /home/grapl/engagement_edge/src/ && \
    zip -g /home/grapl/lambda.zip ./engagement_edge.py && \
    cp /home/grapl/lambda.zip /dist/engagement-edge.zip'

FROM grapl-python-deploy AS grapl-engagement-edge

ARG USERID="1000:1000"
USER $USERID
WORKDIR /home/grapl

COPY --from=engagement-edge-build /home/grapl/venv venv

RUN bash -c ' \
    source venv/bin/activate && \
    chalice new-project app/'

COPY --from=engagement-edge-build /home/grapl/engagement_edge/src/engagement_edge.py app/app.py

CMD bash -c ' \
    source venv/bin/activate && \
    cd /home/grapl/app && \
    chalice local --no-autoreload --host=0.0.0.0 --port=8900'

#
# dgraph-ttl
#

FROM grapl-python-libs AS grapl-dgraph-ttl-build

ARG USERID="1000:1000"
USER $USERID
WORKDIR /home/grapl

COPY --chown=$USERID python/grapl-dgraph-ttl dgraph-ttl

RUN bash -c ' \
    source venv/bin/activate && \
    pip install -r dgraph-ttl/requirements.txt'

CMD bash -c ' \
    cd venv/lib/python3.7/site-packages && \
    zip --quiet -9r /home/grapl/lambda.zip . && \
    cd /home/grapl/dgraph-ttl && \
    zip -g /home/grapl/lambda.zip app.py && \
    cp /home/grapl/lambda.zip /dist/dgraph-ttl.zip'

FROM grapl-python-deploy AS grapl-dgraph-ttl

ARG USERID="1000:1000"
USER $USERID
WORKDIR /home/grapl

COPY --from=grapl-dgraph-ttl-build /home/grapl/venv venv
COPY --from=grapl-dgraph-ttl-build /home/grapl/dgraph-ttl app

CMD bash -c ' \
    source venv/bin/activate && \
    cd /home/grapl/app && \
    chalice local --no-autoreload --host=0.0.0.0 --port=8124'

#
# model-plugin-deployer
#

FROM grapl-python-libs AS grapl-model-plugin-deployer-build

ARG USERID="1000:1000"
USER $USERID
WORKDIR /home/grapl

COPY --chown=$USERID python/grapl-model-plugin-deployer model-plugin-deployer

RUN bash -c ' \
    source venv/bin/activate && \
    cd model-plugin-deployer && \
    pip install .'

CMD bash -c ' \
    cd venv/lib/python3.7/site-packages/ && \
    zip --quiet -9r /home/grapl/lambda.zip ./ && \
    cd /home/grapl/model-plugin-deployer/src/ && \
    zip -g /home/grapl/lambda.zip ./grapl_model_plugin_deployer.py && \
    cp /home/grapl/lambda.zip /dist/model-plugin-deployer.zip'

FROM grapl-python-deploy AS grapl-model-plugin-deployer

ARG USERID="1000:1000"
USER $USERID
WORKDIR /home/grapl

COPY --from=grapl-model-plugin-deployer-build /home/grapl/venv venv

RUN bash -c ' \
    source venv/bin/activate && \
    chalice new-project app/'

COPY --from=grapl-model-plugin-deployer-build /home/grapl/model-plugin-deployer/src/grapl_model_plugin_deployer.py app/app.py

CMD bash -c ' \
    source venv/bin/activate && \
    cd /home/grapl/app && \
    chalice local --no-autoreload --host=0.0.0.0 --port=8123'

#
# grapl-provision
#

FROM grapl-python-libs AS grapl-provision-build

ARG USERID="1000:1000"
USER $USERID
WORKDIR /home/grapl

COPY --chown=$USERID python/grapl_provision ./grapl_local_provision

CMD bash -c ' \
    source venv/bin/activate && \
    python /home/grapl/grapl_local_provision/provision_local_identity_table.py && \
    python /home/grapl/grapl_local_provision/grapl_provision.py \
    '