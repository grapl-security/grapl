FROM python:3.7
RUN apt-get update && apt-get install -y zip
WORKDIR /lambda

ADD requirements.txt /tmp
RUN pip install --quiet -t /lambda -r /tmp/requirements.txt


ADD src/ /src/
RUN python -m compileall -q /lambda

RUN pip install -r /tmp/requirements.txt
