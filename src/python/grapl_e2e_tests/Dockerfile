FROM grapl/grapl-python-build:latest AS grapl-e2e-tests-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . grapl_e2e_tests
COPY --from=grapl/grapl-analyzerlib-python-build /home/grapl/venv venv

COPY --from=grapl/grapl-common-python-build /home/grapl/grapl_common grapl_common
#RUN /bin/bash -c "source venv/bin/activate && cd grapl_e2e_tests && python ./main.py"
# we do the run in the dobi.yaml
