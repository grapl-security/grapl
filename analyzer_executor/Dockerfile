FROM python:3.7
RUN apt-get update && apt-get install --upgrade -y zip
WORKDIR /lambda

# Add the requiremts
ADD requirements.txt /tmp
RUN pip install --upgrade --quiet -t /lambda -r /tmp/requirements.txt 
# Add your source code
ADD src/ /lambda/

# compile the lot.
RUN python -m compileall -q /lambda

RUN zip --quiet -9r /lambda.zip .

FROM scratch
COPY --from=0 /lambda.zip /
