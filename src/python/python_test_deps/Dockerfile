FROM grapl/grapl-python-build:latest AS python-testing-deps
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . python_test_deps
RUN /bin/bash -c "python_test_deps/download_requirements.sh"