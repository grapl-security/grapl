FROM debian:buster-slim AS formatter

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        netbase \
        curl \
        make \
    && rm -rf /var/lib/apt/lists/*

# Install npm, Prettier
RUN curl -fsSL https://deb.nodesource.com/setup_16.x | bash -
RUN apt-get install -y nodejs
RUN npm install -g prettier@2.2.1

USER nobody

WORKDIR /home/linter
COPY ./Makefile /home/linter/Makefile