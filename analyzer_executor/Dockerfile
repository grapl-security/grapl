FROM python:3.7
RUN apt-get update && apt-get install -y zip
WORKDIR /lambda

# Add the requiremts
ADD requirements.txt /tmp
RUN pip install --quiet -t /lambda -r /tmp/requirements.txt 
#&& \
#    find /lambda -type d | xargs chmod ugo+rx && \
 #   find /lambda -type f | xargs chmod ugo+r

# Add your source code
ADD src/ /lambda/
#RUN find /lambda -type d | xargs chmod ugo+rx && \
#    find /lambda -type f | xargs chmod ugo+r

# compile the lot.
RUN python -m compileall -q /lambda

RUN zip --quiet -9r /lambda.zip .

FROM scratch
COPY --from=0 /lambda.zip /
