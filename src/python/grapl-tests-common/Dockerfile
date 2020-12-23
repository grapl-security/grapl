FROM grapl/grapl-python-build:latest AS grapl-tests-common-python-build
USER grapl
WORKDIR /home/grapl

# Steal a mostly-populated venv
COPY --from=grapl/grapl-analyzerlib-python-build /home/grapl/venv venv

# Install test requirements
COPY --from=grapl/python-test-deps /home/grapl/python_test_deps python_test_deps
RUN /bin/bash -c "python_test_deps/install_requirements.sh"

# Install this library into the venv
COPY --chown=grapl . grapl-tests-common
RUN /bin/bash -c "source venv/bin/activate && cd grapl-tests-common && pip install ."
RUN /bin/bash -c "source venv/bin/activate && cd grapl-tests-common && python setup.py sdist bdist_wheel"

# Consumers of this library should then COPY this venv.