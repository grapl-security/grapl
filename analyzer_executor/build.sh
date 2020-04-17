#!/usr/bin/env bash
docker build -t my-lambda .
ID=$(docker create my-lambda /bin/true)		# create a container from the image
docker cp $ID:/ ./		# copy the file from the /
mv ./lambda.zip ./analyzer-executor.zip
cp ./analyzer-executor.zip ../grapl-cdk/
rm -rf ./app;
rm -rf ./bin;
rm -rf ./boot;
rm -rf ./dev/;
rm -rf ./etc/;
rm -rf ./home;
rm -rf ./lib64;
rm -rf ./lib;
rm -rf ./media;
rm -rf ./mnt;
rm -rf ./opt;
rm -rf ./proc/;
rm -rf ./root;
rm -rf ./run;
rm -rf ./sbin;
rm -rf ./srv;
rm -rf ./sys/;
rm -rf ./tmp;
rm -rf ./usr;
rm -rf ./var;
rm ./analyzer-executor.zip
date
