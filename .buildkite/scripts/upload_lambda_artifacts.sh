#!/usr/bin/env bash

set -euo pipefail

source .buildkite/scripts/lib/artifacts.sh
source .buildkite/scripts/lib/cloudsmith.sh
source .buildkite/scripts/lib/lambda.sh
source .buildkite/scripts/lib/version.sh

# We will collect the names of all the lambda artifacts that were
# built in the current pipeline run.
lambda_artifacts=()

echo "--- :buildkite: Retrieving all Lambda ZIP files"
if (buildkite-agent artifact download "dist/*${LAMBDA_SUFFIX}" .); then
    # All versions are the same, as it is dependent on the specific
    # commit we're processing.
    version="$(timestamp_and_sha_version)"

    # Upload each zip file to Cloudsmith and record the name of the
    # lambda function.
    for zip in dist/*"${LAMBDA_SUFFIX}"; do
        echo "--- :cloudsmith: Uploading ${zip} version ${version}"
        # TODO: "raw" may need to change; not sure it's the best
        # name. Also, we may want to use a helper function to
        # determine the appropriate repository based on the pipeline.
        upload_raw_file_to_cloudsmith "${zip}" "${version}" raw
        function_name="$(lambda_name_from_zip "${zip}")"
        lambda_artifacts+=("${function_name}")
    done

    # Generate artifacts JSON file for lambda zip files
    mkdir "${ARTIFACT_FILE_DIRECTORY}"
    artifact_json "${version}" "${lambda_artifacts[@]}" > "${ARTIFACT_FILE_DIRECTORY}/${LAMBDA_ARTIFACTS_FILE}"
else
    echo "^^^ +++" # Un-collapses this section in Buildkite, making it more obvious we couldn't download
    echo "No artifacts to upload"
fi
