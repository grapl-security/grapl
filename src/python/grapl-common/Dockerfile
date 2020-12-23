FROM grapl/grapl-python-build:latest
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . grapl_common
COPY --from=grapl/grapl-python-build /home/grapl/venv venv
RUN /bin/bash -c "source venv/bin/activate && cd grapl_common && pip install ."
RUN /bin/bash -c "source venv/bin/activate && cd grapl_common && python setup.py sdist bdist_wheel"
