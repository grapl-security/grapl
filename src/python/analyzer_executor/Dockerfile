FROM grapl/grapl-analyzerlib-python-build:latest AS analyzer-executor-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . analyzer_executor
COPY --from=grapl/grapl-analyzerlib-python-build /home/grapl/venv venv
# RUN python setup.py test
RUN /bin/bash -c "source venv/bin/activate && cd analyzer_executor && pip install ."
RUN zip --quiet -9r lambda.zip venv/lib/python3.7/site-packages/
RUN zip -g lambda.zip analyzer_executor/src/analyzer-executor.py

FROM grapl/grapl-python-deploy AS grapl-analyzer-executor
USER grapl
WORKDIR /home/grapl
COPY --from=analyzer-executor-build /home/grapl/lambda.zip lambda.zip
COPY --from=analyzer-executor-build /home/grapl/venv venv
COPY --from=analyzer-executor-build /home/grapl/analyzer_executor analyzer_executor
