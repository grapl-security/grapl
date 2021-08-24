# Every time we bring up Localstack, it goes and downloads Elasticmq.
# This "fork" just preloads it in the right spot.
# Saves us time on bringing up Localstack, 
# and avoids an issue wimax@ ran into when running under Nomad.

FROM localstack/localstack-full:0.12.17 AS localstack-base
RUN curl -O /opt/code/localstack/localstack/infra/elasticmq/elasticmq-server.jar \
    https://s3-eu-west-1.amazonaws.com/softwaremill-public/elasticmq-server-1.1.0.jar
     