/**
 * TODO: Move this to some shared library accessible by all Typescript services
 */

import * as aws_types from "@aws-sdk/types";

type Constructable<T> = {
    new (...args: any[]): T;
};

export function getAwsClient<T>(clientType: Constructable<T>): T {
    if ("GRAPL_AWS_ENDPOINT" in process.env) {
        // Running locally
        console.debug("Creating a local client");
        const credentials: aws_types.Credentials = {
            accessKeyId: process.env.GRAPL_AWS_ACCESS_KEY_ID,
            secretAccessKey: process.env.GRAPL_AWS_ACCESS_KEY_SECRET,
        };
        const endpoint = process.env.GRAPL_AWS_ENDPOINT;
        const region = process.env.AWS_DEFAULT_REGION || process.env.AWS_REGION;
        return new clientType({
            endpoint: endpoint,
            credentials: credentials,
            region: region,
        });
    } else {
        // Running on AWS
        return new clientType({});
    }
}
