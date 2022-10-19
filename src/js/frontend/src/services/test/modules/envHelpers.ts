import * as aws_types from "@aws-sdk/types";

type Constructable<T> = {
  new (...args: any[]): T;
};

export default function getAwsClient<T>(clientType: Constructable<T>): T {
  if ("GRAPL_AWS_ENDPOINT" in process.env) {
    // Running locally
    console.debug("Creating a local client");
    const credentials: aws_types.Credentials = {
      accessKeyId: process.env.GRAPL_AWS_ACCESS_KEY_ID ?? "",
      secretAccessKey: process.env.GRAPL_AWS_ACCESS_KEY_SECRET ?? "",
    };

    const endpoint = process.env.GRAPL_AWS_ENDPOINT;
    const region = process.env.AWS_DEFAULT_REGION || process.env.AWS_REGION;

    const username = process.env.GRAPL_TEST_USER_NAME;
    const password = process.env.GRAPL_TEST_USER_PASSWORD_SECRET_ID;

    return new clientType({
      endpoint: endpoint,
      credentials: credentials,
      region: region,
      username: username,
      password: password,
    });
  } else {
    // Running on AWS
    return new clientType({});
  }
}
