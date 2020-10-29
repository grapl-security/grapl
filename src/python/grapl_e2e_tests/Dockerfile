FROM grapl/grapl-python-build:latest AS grapl-e2e-tests-build
CMD "/bin/bash"
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . grapl_e2e_tests
# Expose the `etc/` dir with all its test data in /home/grapl/etc
COPY --from=grapl/etc-build /home/grapl/etc etc

COPY --from=grapl/grapl-tests-common-python-build /home/grapl/venv venv
RUN /bin/bash -c "source venv/bin/activate && pip install zstd"