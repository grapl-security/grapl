FROM grapl/grapl-python-lambda-builder AS engagement-creator-build
WORKDIR /lambda
COPY ./requirements.txt /tmp
RUN pip install --quiet -t /lambda -r /tmp/requirements.txt
COPY ./src/ /lambda/
RUN python -m compileall -q /lambda
RUN zip --quiet -9r /lambda.zip .

FROM grapl/grapl-python-lambda-builder AS grapl-engagement-creator
COPY ./requirements.txt /tmp
RUN pip install -r /tmp/requirements.txt
WORKDIR /app
COPY ./src/ /src/
COPY --from=engagement-creator-build /lambda.zip /
