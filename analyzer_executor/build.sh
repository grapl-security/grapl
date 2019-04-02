#!/usr/bin/env bash
docker build -t my-lambda .
ID=$(docker create my-lambda /bin/true)		# create a container from the image
docker cp $ID:/ ./		# copy the file from the /
mv ./lambda.zip ./analyzer-executor.zip
cp ./analyzer-executor.zip ../grapl-cdk/
rm -rf ./dev/
rm -rf ./etc/
rm -rf ./proc/
rm -rf ./sys/
rm ./analyzer-executor.zip
date
