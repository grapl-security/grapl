FROM grapl/grapl-python-lambda-builder AS engagement-edge-builder
WORKDIR /lambda
COPY ./requirements.txt /tmp
RUN pip install --quiet -t /lambda -r /tmp/requirements.txt
COPY ./src/ /lambda/
RUN python -m compileall -q /lambda
RUN zip --quiet -9r /lambda.zip .

FROM grapl/grapl-python-lambda-builder AS grapl-engagement-edge
COPY ./requirements.txt /tmp
RUN pip install -r /tmp/requirements.txt
RUN chalice new-project app/
WORKDIR /app
COPY ./src/engagement_edge.py /app/app.py
COPY --from=engagement-edge-builder /lambda.zip /
