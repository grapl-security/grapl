#
# graplctl
#
# This image is intended to be used to execute graplctl in integration and
# end-to-end tests. It is unsuitable for other purposes, primarily because it
# contains the following fake AWS credentials:
#
# /home/grapl/.aws/credentials:
# [default]
# aws_access_key_id=fake
# aws_secret_access_key=fake
#
################################################################################

FROM python:3.7-slim-buster AS graplctl
RUN adduser \
    --disabled-password \
    --gecos '' \
    --home /home/grapl \
    --shell /bin/bash \
    grapl
USER grapl
ENV USER=grapl
WORKDIR /home/grapl
RUN mkdir -p /home/grapl/bin
RUN mkdir -p /home/grapl/.aws
RUN echo "[default]\naws_access_key_id=fake\naws_secret_access_key=fake" > /home/grapl/.aws/credentials
COPY --chown=grapl ./bin/graplctl /home/grapl/bin/graplctl
ENV PATH=/home/grapl/bin:$PATH
