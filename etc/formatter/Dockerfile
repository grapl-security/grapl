FROM debian:buster-slim AS formatter

RUN --mount=type=cache,target=/var/lib/apt/lists \
    apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        netbase \
        curl

# Install npm, Prettier
RUN curl -fsSL https://deb.nodesource.com/setup_16.x | bash -
RUN apt-get install -y nodejs
RUN npm install -g prettier@2.2.1
RUN npm install -g prettier-plugin-toml@0.3.1

# TODO:
# In later iterations, we can also install all the Rustup cargofmt stuff here
# as well as Pants to perform containerized linting.
