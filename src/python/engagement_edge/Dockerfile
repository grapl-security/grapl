FROM grapl/grapl-python-build:latest AS engagement-edge-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . engagement_edge
COPY --from=grapl/grapl-analyzerlib-python-build /home/grapl/venv venv
RUN /bin/bash -c "source venv/bin/activate && cd engagement_edge && pip install ."
RUN cd venv/lib/python3.7/site-packages/ && zip --quiet -9r ~/lambda.zip ./
RUN cd engagement_edge/src/ && /bin/bash -c "zip -g ~/lambda.zip ./**/*.py"
RUN mkdir -p dist/engagement-edge && cp lambda.zip dist/engagement-edge/lambda.zip

# Motivation for a different `-test` image:
# I'd like to install stuff into the virtualenv that is only used for tests
# and quality checks, but not deployed to AWS.
FROM engagement-edge-build AS engagement-edge-test
USER grapl
WORKDIR /home/grapl
# Steal and install grapl-tests-common
COPY --from=grapl/grapl-tests-common-python-build /home/grapl/grapl-tests-common grapl-tests-common
RUN /bin/bash -c "source venv/bin/activate && cd grapl-tests-common && pip install ."
# Install typechecking stuff
RUN /bin/bash -c "source venv/bin/activate && cd engagement_edge && pip install .[typecheck]"

FROM grapl/grapl-python-deploy AS grapl-engagement-edge
USER root
RUN apt-get update && apt-get install -y wait-for-it
USER grapl
WORKDIR /home/grapl
COPY --from=engagement-edge-build /home/grapl/lambda.zip lambda.zip
COPY --from=engagement-edge-build /home/grapl/venv venv
RUN /bin/bash -c "source venv/bin/activate && chalice new-project app/"
COPY --from=engagement-edge-build /home/grapl/engagement_edge/src/engagement_edge.py app/app.py
