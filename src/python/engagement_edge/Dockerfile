FROM grapl/grapl-graph-descriptions-python-build AS engagement-edge-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . engagement_edge
COPY --from=grapl/grapl-graph-descriptions-python-build /home/grapl/venv venv
# RUN python setup.py test
RUN source venv/bin/activate && cd engagement_edge && pip install .
RUN zip --quiet -9r lambda.zip venv/lib/python3.7/site-packages/
RUN zip -g lambda.zip engagement_edge/src/engagement_edge.py

FROM python:3.7-alpine AS grapl-engagement-edge
RUN adduser -D --gecos '' --home /home/grapl --shell /bin/bash grapl
USER grapl
ENV USER grapl
WORKDIR /home/grapl
COPY --from=engagement-edge-build /home/grapl/lambda.zip lambda.zip
COPY --from=engagement-edge-build /home/grapl/venv venv
RUN source venv/bin/activate && chalice new-project app/
COPY --from=engagement-edge-build /home/grapl/engagement_edge/src/engagement_edge.py app/app.py
