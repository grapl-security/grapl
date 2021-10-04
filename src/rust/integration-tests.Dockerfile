# syntax=docker.io/docker/dockerfile:1.3-labs
# Using labs syntax for here-document support.
# https://github.com/moby/buildkit/blob/master/frontend/dockerfile/docs/syntax.md#here-documents

# image for running Rust integration tests

FROM debian:bullseye-slim

COPY tests /tests

# Create simple test harness to run all tests and report failure
RUN <<EOF cat > /run-tests.sh
#!/bin/sh
EXIT_STATUS=0
for test in \$(find /tests -maxdepth 1 -type f -executable -exec readlink -f {} \;)
do
    echo Executing "\${test}"
    "\${test}" || EXIT_STATUS=\$?
done
exit \${EXIT_STATUS}
EOF
RUN chmod +x run-tests.sh

USER nobody

ENTRYPOINT [ "/run-tests.sh" ]