FROM grapl/grapl-graph-descriptions-python-build:latest AS grapl-analyzerlib-python-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . grapl_analyzerlib
COPY --from=grapl/grapl-graph-descriptions-python-build /home/grapl/venv venv
RUN /bin/bash -c "source venv/bin/activate && cd grapl_analyzerlib && pip install ."
RUN /bin/bash -c "source venv/bin/activate && cd grapl_analyzerlib && python setup.py sdist bdist_wheel"
