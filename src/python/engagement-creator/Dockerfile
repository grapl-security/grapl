FROM grapl/grapl-graph-descriptions-python-build AS engagement-creator-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . engagement-creator
COPY --from=grapl/grapl-graph-descriptions-python-build /home/grapl/venv venv
# RUN python setup.py test
RUN source venv/bin/activate && cd engagement-creator && pip install .
RUN zip --quiet -9r lambda.zip venv/lib/python3.7/site-packages/
RUN zip -g lambda.zip engagement-creator/src/engagement-creator.py

FROM python:3.7-alpine AS grapl-engagement-creator
RUN adduser -D --gecos '' --home /home/grapl --shell /bin/bash grapl
USER grapl
ENV USER grapl
WORKDIR /home/grapl
COPY --from=engagement-creator-build /home/grapl/lambda.zip lambda.zip
COPY --from=engagement-creator-build /home/grapl/venv venv
COPY --from=engagement-creator-build /home/grapl/engagement-creator engagement-creator
