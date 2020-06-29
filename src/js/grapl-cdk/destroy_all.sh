#!/usr/bin/env bash
for BUCKET in $(aws s3 ls s3:// | cut -d' ' -f 3)
do
    aws s3 rm --recursive s3://$BUCKET
done

npm run build &&
cdk destroy -f --require-approval=never "*"

date
