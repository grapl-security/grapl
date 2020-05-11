FROM grapl/grapl-python-lambda-builder AS analyzer-executor-build
WORKDIR /lambda
COPY ./requirements.txt /tmp
RUN pip install --quiet -t /lambda -r /tmp/requirements.txt
COPY ./src/ /lambda/
RUN python -m compileall -q /lambda
RUN zip --quiet -9r /lambda.zip .

FROM grapl/grapl-python-lambda-builder AS grapl-analyzer-executor
COPY ./requirements.txt /tmp
RUN pip install -r /tmp/requirements.txt
WORKDIR /src
COPY ./src/ /src/
COPY --from=analyzer-executor-build /lambda.zip /
