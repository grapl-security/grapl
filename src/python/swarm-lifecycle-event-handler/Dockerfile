FROM grapl/grapl-python-build:latest AS swarm-lifecycle-event-handler-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . ./swarm-lifecycle-event-handler
COPY --from=grapl/grapl-python-build /home/grapl/venv venv
RUN /bin/bash -c "source venv/bin/activate && pip install -r swarm-lifecycle-event-handler/requirements.txt"
# build AWS Lambda deploy artifact
# first, pull in the virtualenv's site-packages
RUN cd venv/lib/python3.7/site-packages && zip --quiet -9r ../../../../lambda.zip .
# then add the chalice app.py as the main entrypoint
RUN cd swarm-lifecycle-event-handler && zip -g ../lambda.zip app.py
# finally copy out the artifact to the dist mount
RUN mkdir -p dist/swarm-lifecycle-event-handler && cp lambda.zip dist/swarm-lifecycle-event-handler/lambda.zip
