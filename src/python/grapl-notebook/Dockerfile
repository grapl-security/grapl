FROM grapl/grapl-python-build:latest AS grapl-notebook
USER grapl
SHELL ["/bin/bash", "-o", "pipefail", "-c"]
EXPOSE 8888
WORKDIR /home/grapl
# Set up virtualenv
COPY --from=grapl/grapl-analyzerlib-python-build /home/grapl/venv venv
RUN /bin/bash -c "source venv/bin/activate && pip install jupyter"

# Set up jupyter-notebook stuff
RUN mkdir -p /home/grapl/grapl-notebook/model_plugins
COPY --chown=grapl ./jupyter_notebook_config.py /home/grapl/.jupyter/
COPY --chown=grapl ./Demo_Engagement.ipynb /home/grapl/grapl-notebook/

# Run it
CMD source venv/bin/activate && cd grapl-notebook && jupyter notebook --ip="0.0.0.0"