ARG TAG=latest

#
# e2e-tests
################################################################################
FROM grapl/graplctl:$TAG AS e2e-tests
USER root
RUN apt-get update && apt-get install -y --no-install-recommends unzip
USER grapl
COPY --chown=grapl ./dist/src.python.e2e-test-runner.e2e_test_runner/lambda.zip lambda.zip
RUN unzip lambda.zip
