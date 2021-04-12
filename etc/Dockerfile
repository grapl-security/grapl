ARG TAG=latest
# The point of this Dockerfile is simply to expose the
# local_grapl scripts and sample data to grapl_e2e_tests.

# doesn't have to be node alpine, but eh, it's minimal and it works
FROM grapl/grapl-base:$TAG AS etc-build
WORKDIR /home/grapl
COPY . etc
