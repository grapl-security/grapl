FROM jupyter/minimal-notebook

COPY --chown=jovyan:users grapl_analyzerlib/grapl_analyzerlib /home/jovyan/grapl_analyzerlib
COPY --chown=jovyan:users grapl-common/grapl_common /home/jovyan/grapl_common

RUN pip install pydgraph

