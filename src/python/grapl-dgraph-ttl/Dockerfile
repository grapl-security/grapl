FROM grapl/grapl-analyzerlib-python-build:latest AS grapl-dgraph-ttl-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . ./grapl-dgraph-ttl
COPY --from=grapl/grapl-analyzerlib-python-build /home/grapl/venv venv
RUN /bin/bash -c "source venv/bin/activate && pip install -r grapl-dgraph-ttl/requirements.txt"
RUN zip --quiet -9r lambda.zip venv/lib/python3.7/site-packages/
RUN zip -g lambda.zip grapl-dgraph-ttl/app.py

FROM grapl/grapl-python-deploy AS grapl-dgraph-ttl
USER grapl
WORKDIR /home/grapl
COPY --from=grapl-dgraph-ttl-build /home/grapl/lambda.zip lambda.zip
COPY --from=grapl-dgraph-ttl-build /home/grapl/venv venv
COPY --from=grapl-dgraph-ttl-build /home/grapl/grapl-dgraph-ttl app
