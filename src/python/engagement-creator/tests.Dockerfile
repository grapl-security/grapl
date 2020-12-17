FROM grapl/grapl-python-build:latest AS engagement-creator-test
USER grapl
WORKDIR /home/grapl

# Grab venv
COPY --from=grapl/engagement-creator-build /home/grapl/venv venv

# Install test requirements
COPY --from=grapl/python-test-deps /home/grapl/python_test_deps python_test_deps
RUN /bin/bash -c "python_test_deps/install_requirements.sh"

# Grab and run source
COPY --from=grapl/engagement-creator-build /home/grapl/engagement-creator engagement-creator
RUN /bin/bash -c "source venv/bin/activate && cd engagement-creator && pip install .[typecheck]"