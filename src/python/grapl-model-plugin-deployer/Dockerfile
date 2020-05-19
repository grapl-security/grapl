FROM grapl/grapl-graph-descriptions-python-build AS grapl-model-plugin-deployer-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . grapl-model-plugin-deployer
COPY --from=grapl/grapl-graph-descriptions-python-build /home/grapl/venv venv
# RUN python setup.py test
RUN source venv/bin/activate && cd grapl-model-plugin-deployer && pip install .
RUN zip --quiet -9r lambda.zip venv/lib/python3.7/site-packages/
RUN zip -g lambda.zip grapl-model-plugin-deployer/src/grapl_model_plugin_deployer.py

FROM python:3.7-alpine AS grapl-model-plugin-deployer
RUN adduser -D --gecos '' --home /home/grapl --shell /bin/bash grapl
USER grapl
ENV USER grapl
WORKDIR /home/grapl
COPY --from=grapl-model-plugin-deployer-build /home/grapl/lambda.zip lambda.zip
COPY --from=grapl-model-plugin-deployer-build /home/grapl/venv venv
RUN source venv/bin/activate && chalice new-project app/
COPY --from=grapl-model-plugin-deployer-build /home/grapl/grapl-model-plugin-deployer/src/grapl_model_plugin_deployer.py app/app.py
