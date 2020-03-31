FROM python:3.7
RUN apt-get update && apt-get install -y zip
WORKDIR /lambda

ADD requirements.txt /tmp
RUN pip install --quiet -t /lambda -r /tmp/requirements.txt


ADD src/ /lambda/
RUN python -m compileall -q /lambda

RUN zip --quiet -9r /lambda.zip .
FROM scratch
COPY --from=0 /lambda.zip /

FROM python:3.7
RUN apt-get update && apt-get install -y zip

ADD requirements.txt /tmp
RUN pip install -r /tmp/requirements.txt
RUN pip install chalice

RUN chalice new-project app/
WORKDIR /app
VOLUME /app

ADD src/engagement_edge.py /app/app.py

