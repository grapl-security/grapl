# The point of this Dockerfile is simply to expose the
# local_grapl scripts and sample data to grapl_e2e_tests.

# doesn't have to be node alpine, but eh, it's minimal and it works
FROM grapl/grapl-python-build:latest AS grapl-local-provision-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . ./grapl_local_provision
COPY --from=grapl/grapl-analyzerlib-python-build /home/grapl/venv venv