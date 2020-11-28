FROM grapl/engagement-edge-build AS engagement-edge-test
USER grapl
WORKDIR /home/grapl

COPY --from=grapl/python-test-deps /home/grapl/python_test_deps python_test_deps
RUN /bin/bash -c "python_test_deps/install_requirements.sh"

# Steal and install grapl-tests-common
COPY --from=grapl/grapl-tests-common-python-build /home/grapl/grapl-tests-common grapl-tests-common
RUN /bin/bash -c "source venv/bin/activate && cd grapl-tests-common && pip install ."
