FROM grapl/grapl-python-build:latest AS grapl-tests-common-python-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . grapl-tests-common
COPY --from=grapl/grapl-analyzerlib-python-build /home/grapl/venv venv
COPY --from=grapl/etc-build /home/grapl/etc etc
RUN /bin/bash -c "source venv/bin/activate && cd grapl-tests-common && pip install ."
RUN /bin/bash -c "source venv/bin/activate && cd grapl-tests-common && python setup.py sdist bdist_wheel"
