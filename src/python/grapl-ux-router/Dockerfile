FROM grapl/grapl-python-build:latest AS grapl-ux-router-build
USER grapl
WORKDIR /home/grapl
COPY --chown=grapl . grapl-ux-router
COPY --from=grapl/grapl-analyzerlib-python-build /home/grapl/venv venv
RUN /bin/bash -c "source venv/bin/activate && cd grapl-ux-router && pip install ."
RUN cd venv/lib/python3.7/site-packages/ && zip --quiet -9r ~/lambda.zip ./
RUN cd grapl-ux-router/src/ && /bin/bash -c "zip -g ~/lambda.zip ./**/*.py"
RUN mkdir -p dist/grapl-ux-router && cp lambda.zip dist/grapl-ux-router/lambda.zip

FROM grapl/grapl-python-deploy AS grapl-grapl-ux-router
USER root
RUN apt-get update && apt-get install -y wait-for-it
USER grapl
WORKDIR /home/grapl
COPY --from=grapl-ux-router-build /home/grapl/lambda.zip lambda.zip
COPY --from=grapl-ux-router-build /home/grapl/venv venv
RUN /bin/bash -c "source venv/bin/activate && chalice new-project app/"
COPY --from=grapl-ux-router-build /home/grapl/grapl-ux-router/src/grapl_ux_router.py app/app.py
