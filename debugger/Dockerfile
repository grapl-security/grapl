FROM ubuntu:21.04

# Feel free to add whatever convenient testing utilities you like.
RUN apt-get update \
    && apt-get install --yes --no-install-recommends \
       curl=7.74.0-1ubuntu2.3 \
       htop=3.0.5-6 \
       iputils-ping=3:20210202-1 \
       jq=1.6-2.1ubuntu1 \
       net-tools=1.60+git20181103.0eebece-1ubuntu2 \
       nmap=7.91+dfsg1+really7.80+dfsg1-1 \
       strace=5.11-0ubuntu1 \
       wget=1.21-1ubuntu3 \
    && rm -rf /var/lib/apt/lists/*
