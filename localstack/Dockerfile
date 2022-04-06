# syntax=docker/dockerfile:1.3-labs

# Every time we bring up Localstack, it goes and downloads Elasticmq.
# This "fork" just preloads the JAR in the right spot.
# Due to https://github.com/localstack/localstack/issues/5821

FROM localstack/localstack-light:0.14.2 AS localstack-base

RUN <<EOF
mkdir -p /opt/code/localstack/localstack/infra/elasticmq/
URL="https://s3-eu-west-1.amazonaws.com/softwaremill-public/elasticmq-server-1.1.0.jar"
OUTPUT="/opt/code/localstack/localstack/infra/elasticmq/elasticmq-server.jar"
curl --location --output "${OUTPUT}" "${URL}"
EOF
