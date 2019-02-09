docker build -t my-lambda .
ID=$(docker create my-lambda /bin/true)		# create a container from the image
docker cp $ID:/ ./		# copy the file from the /
mv ./lambda.zip ./engagement-creator.zip
cp ./engagement-creator.zip ../grapl-cdk/
rm -rf ./dev/
rm -rf ./etc/
rm -rf ./proc/
rm -rf ./sys/
date
