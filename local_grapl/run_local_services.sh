docker run -d -p 9324:9324 vsouza/sqs-local;
docker run -p 9000:9000 minio/minio server /data &

docker run --rm -p 8000:8000 -p 8080:8080 -p 9080:9080 dgraph/standalone:latest &
docker run --rm -p 8001:8000 -p 8081:8080 -p 9081:9080 dgraph/standalone:latest &
