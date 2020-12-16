# Motivation for a different `-test` image:
# I'd like to install stuff into the virtualenv that is only used for tests
# and quality checks, but not deployed to AWS.

# And you can't define this image in the same Dockerfile, because:
# dobi evaluates the full Dockerfile when building the first stage of the image; and if
# grapl-tests-common isn't built yet (as it need not be for engagement-edge-build), it'll fail.

FROM grapl/engagement-edge-build AS engagement-edge-test
USER grapl
WORKDIR /home/grapl
# Steal and install grapl-tests-common
COPY --from=grapl/grapl-tests-common-python-build /home/grapl/grapl-tests-common grapl-tests-common
RUN /bin/bash -c "source venv/bin/activate && cd grapl-tests-common && pip install ."
# Install typechecking stuff
RUN /bin/bash -c "source venv/bin/activate && cd engagement_edge && pip install .[typecheck]"
