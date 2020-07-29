FROM grapl/grapl-analyzerlib-python-build:latest AS grapl-dgraph-ttl-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . ./dgraph-ttl
COPY --from=grapl/grapl-analyzerlib-python-build /home/grapl/venv venv
RUN /bin/bash -c "source venv/bin/activate && pip install -r dgraph-ttl/requirements.txt"
RUN cd venv/lib/python3.7/site-packages && zip --quiet -9r ../../../../lambda.zip .
RUN cd dgraph-ttl && zip -g ../lambda.zip app.py
RUN mkdir -p dist/dgraph-ttl && cp lambda.zip dist/dgraph-ttl/lambda.zip

FROM grapl/grapl-python-deploy AS grapl-dgraph-ttl
USER grapl
WORKDIR /home/grapl
COPY --from=grapl-dgraph-ttl-build /home/grapl/lambda.zip lambda.zip
COPY --from=grapl-dgraph-ttl-build /home/grapl/venv venv
COPY --from=grapl-dgraph-ttl-build /home/grapl/dgraph-ttl app
