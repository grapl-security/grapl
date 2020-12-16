FROM python:3.7-slim-buster AS grapl-python-build
RUN apt-get update && apt-get -y upgrade && apt-get -y install --no-install-recommends musl-dev protobuf-compiler build-essential zip bash wait-for-it
ENV PROTOC /usr/bin/protoc
ENV PROTOC_INCLUDE /usr/include
RUN adduser --disabled-password --gecos '' --home /home/grapl --shell /bin/bash grapl
USER grapl
ENV USER grapl
WORKDIR /home/grapl
RUN python3 -mvenv venv
RUN /bin/bash -c "source venv/bin/activate && pip install --upgrade pip"
RUN /bin/bash -c "source venv/bin/activate && pip install wheel grpcio chalice"

# no-op the base image, so it doesn't exec python3 as a repl
CMD :