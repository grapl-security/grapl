FROM debian:buster-slim AS format

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        netbase \
        curl \
    && rm -rf /var/lib/apt/lists/*

# Install npm, Prettier
RUN curl -fsSL https://deb.nodesource.com/setup_16.x | bash -
RUN apt-get install -y nodejs
RUN npm install -g prettier@2.2.1

SHELL ["/bin/bash", "-c"]
#USER nobody