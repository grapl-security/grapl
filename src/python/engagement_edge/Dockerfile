FROM grapl/grapl-analyzerlib-python-build:latest AS engagement-edge-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . engagement_edge
COPY --from=grapl/grapl-analyzerlib-python-build /home/grapl/venv venv
RUN /bin/bash -c "source venv/bin/activate && cd engagement_edge && pip install ."
RUN /bin/bash -c "source venv/bin/activate && cd engagement_edge && py.test -n auto -m 'not integration_test'"
RUN cd venv/lib/python3.7/site-packages/ && zip --quiet -9r ~/lambda.zip ./
RUN cd engagement_edge/src/ && zip -g ~/lambda.zip ./engagement_edge.py
RUN mkdir -p dist/engagement-edge && cp lambda.zip dist/engagement-edge/lambda.zip

FROM grapl/grapl-python-deploy AS grapl-engagement-edge
USER grapl
WORKDIR /home/grapl
COPY --from=engagement-edge-build /home/grapl/lambda.zip lambda.zip
COPY --from=engagement-edge-build /home/grapl/venv venv
RUN /bin/bash -c "source venv/bin/activate && chalice new-project app/"
COPY --from=engagement-edge-build /home/grapl/engagement_edge/src/engagement_edge.py app/app.py
