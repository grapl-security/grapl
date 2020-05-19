FROM python:3.7-alpine AS grapl-python-build
RUN apk add --no-cache musl-dev libc6-compat protoc build-base linux-headers zip
ENV PROTOC /usr/bin/protoc
ENV PROTOC_INCLUDE /usr/include
RUN adduser -D --gecos '' --home /home/grapl --shell /bin/bash grapl
USER grapl
ENV USER grapl
WORKDIR /home/grapl
RUN python -mvenv venv
RUN source venv/bin/activate && pip install --upgrade pip
RUN source venv/bin/activate && pip install wheel grpcio chalice
