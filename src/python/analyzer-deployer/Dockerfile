FROM grapl/grapl-python-build:latest AS analyzer-deployer-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . ./analyzer-deployer
COPY --from=grapl/grapl-python-build /home/grapl/venv venv
RUN /bin/bash -c "source venv/bin/activate && pip install -r analyzer-deployer/requirements.txt && python -m mypy_boto3"
# build AWS Lambda deploy artifact
# first, pull in the virtualenv's site-packages
RUN cd venv/lib/python3.7/site-packages && zip --quiet -9r ../../../../lambda.zip .
# then add the chalice app.py as the main entrypoint
RUN cd analyzer-deployer/analyzer_deployer && zip -g ../../lambda.zip app.py
# finally copy out the artifact to the dist mount
RUN mkdir -p dist/analyzer-deployer && cp lambda.zip dist/analyzer-deployer/lambda.zip

FROM grapl/grapl-python-deploy AS analyzer-deployer
USER grapl
WORKDIR /home/grapl
COPY --from=analyzer-deployer-build /home/grapl/lambda.zip lambda.zip
COPY --from=analyzer-deployer-build /home/grapl/venv venv
COPY --from=analyzer-deployer-build /home/grapl/analyzer-deployer app
