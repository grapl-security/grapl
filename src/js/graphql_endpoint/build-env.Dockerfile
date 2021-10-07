FROM node:16-buster-slim

SHELL ["/bin/bash", "-c"]

# Manually create Docker volume mount points so we can set the mode
# to make them a+w.
RUN mkdir --mode=777 --parents /graphql_endpoint/{.yarn/state,node_modules}
