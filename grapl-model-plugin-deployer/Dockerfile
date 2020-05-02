FROM grapl/grapl-python-lambda-builder
WORKDIR /lambda

COPY requirements.txt /tmp
RUN pip install --quiet -t /lambda -r /tmp/requirements.txt


COPY src/ /lambda/
RUN python -m compileall -q /lambda

RUN zip --quiet -9r /lambda.zip .
FROM scratch

FROM grapl/grapl-python-lambda-builder

ADD requirements.txt /tmp
RUN pip install -r /tmp/requirements.txt

RUN chalice new-project app/
WORKDIR /app

ADD src/grapl_model_plugin_deployer.py /app/app.py

COPY --from=0 /lambda.zip /

















