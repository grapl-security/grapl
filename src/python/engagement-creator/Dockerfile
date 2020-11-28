FROM grapl/grapl-python-build:latest AS engagement-creator-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . engagement-creator
COPY --from=grapl/grapl-analyzerlib-python-build /home/grapl/venv venv
RUN /bin/bash -c "source venv/bin/activate && cd engagement-creator && pip install ."
RUN cd venv/lib/python3.7/site-packages/ && zip --quiet -9r ~/lambda.zip ./

RUN cd engagement-creator/src/ && zip -v -g ~/lambda.zip engagement-creator.py
RUN mkdir -p dist/engagement-creator && cp lambda.zip dist/engagement-creator/lambda.zip

FROM grapl/grapl-python-deploy:latest AS grapl-engagement-creator
USER grapl
WORKDIR /home/grapl
COPY --from=engagement-creator-build /home/grapl/lambda.zip lambda.zip
COPY --from=engagement-creator-build /home/grapl/venv venv
COPY --from=engagement-creator-build /home/grapl/engagement-creator engagement-creator
