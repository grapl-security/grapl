FROM grapl/grapl-analyzerlib-python-build:latest AS grapl-model-plugin-deployer-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . model-plugin-deployer
COPY --from=grapl/grapl-analyzerlib-python-build /home/grapl/venv venv
RUN /bin/bash -c "source venv/bin/activate && cd model-plugin-deployer && pip install ."
RUN cd venv/lib/python3.7/site-packages/ && zip --quiet -9r ~/lambda.zip ./
RUN cd model-plugin-deployer/src/ && zip -g ~/lambda.zip ./grapl_model_plugin_deployer.py
RUN mkdir -p dist/model-plugin-deployer && cp ~/lambda.zip dist/model-plugin-deployer/lambda.zip

FROM grapl/grapl-python-deploy AS grapl-model-plugin-deployer
USER grapl
WORKDIR /home/grapl
COPY --from=grapl-model-plugin-deployer-build /home/grapl/lambda.zip lambda.zip
COPY --from=grapl-model-plugin-deployer-build /home/grapl/venv venv
RUN /bin/bash -c "source venv/bin/activate && chalice new-project app/"
COPY --from=grapl-model-plugin-deployer-build /home/grapl/model-plugin-deployer/src/grapl_model_plugin_deployer.py app/app.py
