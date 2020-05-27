FROM grapl/grapl-analyzerlib-python-build:latest AS engagement-creator-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . engagement-creator
COPY --from=grapl/grapl-analyzerlib-python-build /home/grapl/venv venv
# RUN python setup.py test
RUN source venv/bin/activate && cd engagement-creator && pip install .
RUN zip --quiet -9r lambda.zip venv/lib/python3.7/site-packages/
RUN zip -g lambda.zip engagement-creator/src/engagement-creator.py

FROM grapl/grapl-python-deploy:latest AS grapl-engagement-creator
USER grapl
WORKDIR /home/grapl
COPY --from=engagement-creator-build /home/grapl/lambda.zip lambda.zip
COPY --from=engagement-creator-build /home/grapl/venv venv
COPY --from=engagement-creator-build /home/grapl/engagement-creator engagement-creator
