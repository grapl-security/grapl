FROM python:3.7-alpine AS grapl-python-deploy
RUN apk add libstdc++
RUN adduser -D --gecos '' --home /home/grapl --shell /bin/bash grapl
USER grapl
ENV USER grapl
