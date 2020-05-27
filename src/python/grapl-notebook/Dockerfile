FROM jupyter/minimal-notebook

RUN pip install --user --upgrade grapl_analyzerlib
COPY ./jupyter_notebook_config.py /home/jovyan/.jupyter/jupyter_notebook_config.py
COPY ./Demo_Engagement.ipynb /home/jovyan/
RUN mkdir -p /home/jovyan/model_plugins

CMD jupyter notebook
