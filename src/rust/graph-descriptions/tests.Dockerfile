FROM grapl/grapl-graph-descriptions-python-build:latest AS grapl-graph-descriptions-test

# Install test requirements
COPY --from=grapl/python-test-deps /home/grapl/python_test_deps python_test_deps
RUN /bin/bash -c "python_test_deps/install_requirements.sh"

# Run tests
RUN /bin/bash -c "source venv/bin/activate && cd graph-descriptions && py.test -n auto -m 'not integration_test'"