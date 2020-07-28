FROM python:3.7-slim-buster AS grapl-python-deploy
RUN apt-get install -y libstdc++6 bash
RUN adduser --disabled-password --gecos '' --home /home/grapl --shell /bin/bash grapl
USER grapl
ENV USER grapl
