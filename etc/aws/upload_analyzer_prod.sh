# no unset variables please
set -eu

ANALYZER_UPLOAD_SCRIPT_PATH=$(dirname "$(readlink -f "$0")")
LOCAL_GRAPL_DIR=$ANALYZER_UPLOAD_SCRIPT_PATH/../local_grapl

aws s3 cp \
    $LOCAL_GRAPL_DIR/suspicious_svchost/main.py \
    s3://${GRAPL_DEPLOYMENT_NAME}-analyzers-bucket/analyzers/suspicious_svchost/main.py

aws s3 cp \
    $LOCAL_GRAPL_DIR/unique_cmd_parent/main.py \
    s3://${GRAPL_DEPLOYMENT_NAME}-analyzers-bucket/analyzers/unique_cmd_parent/main.py