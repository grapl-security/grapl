FROM jupyter/minimal-notebook

COPY --chown=jovyan grapl_analyzerlib grapl_analyzerlib
RUN bash -c ' \
    cd grapl_analyzerlib && \
    pip install . && \
    python setup.py sdist bdist_wheel'

COPY --chown=jovyan grapl-notebook/jupyter_notebook_config.py /home/jovyan/.jupyter/jupyter_notebook_config.py
COPY --chown=jovyan grapl-notebook/Demo_Engagement.ipynb /home/jovyan/
RUN mkdir -p /home/jovyan/model_plugins

CMD jupyter notebook
