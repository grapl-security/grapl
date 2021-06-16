#!/bin/bash -x

set -e

pushd /tmp/files
wget \
	--no-verbose \
	--input-file=urls \
	--no-clobber
sha256sum -c shasums

for zipped in *.zip
do
	unzip "$zipped"
done
