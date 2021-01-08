FROM grapl/analyzer-executor-build

# Install test requirements
COPY --from=grapl/python-test-deps /home/grapl/python_test_deps python_test_deps
RUN /bin/bash -c "python_test_deps/install_requirements.sh"