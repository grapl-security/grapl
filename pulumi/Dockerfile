FROM python:3.7-slim-buster

RUN apt-get update && apt-get install --yes \
    curl \
    wait-for-it

RUN curl -O https://get.pulumi.com/releases/sdk/pulumi-v2.21.2-linux-x64.tar.gz && \
    tar -xzvf pulumi-*.tar.gz
ENV PATH=/pulumi:${PATH}

RUN adduser \
        --disabled-password \
        --gecos '' \
        --home /home/grapl \
        --shell /bin/bash \
        grapl

USER grapl
ENV USER=grapl
WORKDIR /home/grapl

RUN mkdir venv

# Automatically ensures that our virtualenv is created and active on
# all subsequent actions
ENV VIRTUAL_ENV=/home/grapl/venv
RUN python3 -m venv $VIRTUAL_ENV
ENV PATH="$VIRTUAL_ENV/bin:$PATH"

COPY requirements.txt requirements.txt

RUN pip install --upgrade pip && \
    pip install -r requirements.txt

COPY --chown=grapl . .
