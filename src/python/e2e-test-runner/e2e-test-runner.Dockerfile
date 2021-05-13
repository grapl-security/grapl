ARG TAG=latest

#
# e2e-tests
################################################################################
FROM grapl/graplctl:$TAG AS e2e-tests
USER root
RUN apt-get update && apt-get install -y --no-install-recommends unzip
USER grapl
<<<<<<< HEAD
COPY --chown=grapl ./dist/src.python.e2e-test-runner.e2e_test_runner/lambda.zip lambda.zip
=======
COPY --chown=grapl ./dist/src.python.e2e-test-runner.src/lambda.zip lambda.zip
>>>>>>> make provisioner lambda run in localstack and e2e tests run in lambda
RUN unzip lambda.zip
