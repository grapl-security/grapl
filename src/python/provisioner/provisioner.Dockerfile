ARG TAG=latest

#
# provisioner
################################################################################
FROM grapl/graplctl:$TAG AS provisioner
USER root
RUN apt-get update && apt-get install -y --no-install-recommends unzip
USER grapl
COPY --chown=grapl ./dist/src.python.provisioner.provisioner/lambda.zip lambda.zip
RUN unzip lambda.zip
