FROM grapl/grapl-python-build:latest AS grapl-graph-descriptions-python-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . graph-descriptions
COPY --from=grapl/grapl-python-build /home/grapl/venv venv
RUN source venv/bin/activate && cd graph-descriptions && pip install .
