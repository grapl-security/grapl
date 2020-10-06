FROM grapl/grapl-python-build:latest AS grapl-local-provision-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . ./grapl_local_provision
COPY --from=grapl/grapl-analyzerlib-python-build /home/grapl/venv venv