FROM grapl/grapl-analyzerlib-python-build:latest AS analyzer-executor-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . analyzer_executor
COPY --from=grapl/grapl-analyzerlib-python-build /home/grapl/venv venv
RUN /bin/bash -c "source venv/bin/activate && cd analyzer_executor && pip install ."
RUN cd venv/lib/python3.7/site-packages/ && zip --quiet -9r ~/lambda.zip ./
RUN cd analyzer_executor/src/ && zip -g ~/lambda.zip ./analyzer-executor.py
RUN mkdir -p dist/analyzer-executor && cp ~/lambda.zip dist/analyzer-executor/lambda.zip

FROM grapl/grapl-python-deploy AS grapl-analyzer-executor
USER grapl
WORKDIR /home/grapl
COPY --from=analyzer-executor-build /home/grapl/lambda.zip lambda.zip
COPY --from=analyzer-executor-build /home/grapl/venv venv
COPY --from=analyzer-executor-build /home/grapl/analyzer_executor analyzer_executor
