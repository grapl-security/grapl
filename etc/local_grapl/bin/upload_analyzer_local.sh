# no unset variables please
set -eu

analyzer_upload_script_path=$(dirname "$(readlink -f "$0")")
local_grapl_dir="${analyzer_upload_script_path}/../"

# Ensures the environment is set up appropriately for interacting with
# Local Grapl (running inside a Docker Compose network locally) from
# *outside* that network (i.e., from your workstation).
#
# NOTE: These values are copied from local-grapl.env. It's
# unfortunate, yes, but in the interests of a decent
# user-experience, we'll eat that pain for now. In the near term,
# we should pull this functionality into something like graplctl
# with a more formalized way of pointing to a specific Grapl
# instance.
export AWS_ACCESS_KEY_ID="THIS_IS_A_FAKE_AWS_ACCESS_KEY_ID"
export AWS_SECRET_ACCESS_KEY="THIS_IS_A_FAKE_AWS_SECRET_ACCESS_KEY"

bucket_prefix="local-grapl"
local_s3_endpoint="http://localhost:9000"
region="us-east-1"

aws s3 cp \
    "${local_grapl_dir}/suspicious_svchost/main.py" \
    "s3://${bucket_prefix}-analyzers-bucket/analyzers/suspicious_svchost/main.py" \
    --endpoint-url="${local_s3_endpoint}" --region="${region}"

aws s3 cp \
    "${local_grapl_dir}/unique_cmd_parent/main.py" \
    "s3://${bucket_prefix}-analyzers-bucket/analyzers/unique_cmd_parent/main.py" \
    --endpoint-url="${local_s3_endpoint}" --region="${region}"
