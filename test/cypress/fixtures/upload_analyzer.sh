set -eu

export AWS_ACCESS_KEY_ID=minioadmin
export AWS_SECRET_ACCESS_KEY=minioadmin
export BUCKET_PREFIX="local-grapl"
export LOCAL_GRAPL_DIR="./cypress/fixtures"

aws s3 cp \
    $LOCAL_GRAPL_DIR/suspicious_svchost/main.py \
    s3://${BUCKET_PREFIX}-analyzers-bucket/analyzers/suspicious_svchost/main.py \
    --endpoint-url=http://localhost:9000
aws s3 cp \
    $LOCAL_GRAPL_DIR/unique_cmd_parent/main.py \
    s3://${BUCKET_PREFIX}-analyzers-bucket/analyzers/unique_cmd_parent/main.py \
    --endpoint-url=http://localhost:9000