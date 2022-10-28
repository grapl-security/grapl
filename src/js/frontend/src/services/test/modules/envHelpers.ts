import * as aws_types from "@aws-sdk/types";

type Constructable<T> = {
  new (...args: any[]): T;
};

const getAwsClient = <T> (clientType: Constructable<T>): T => {
  if ("GRAPL_AWS_ENDPOINT" in process.env) {
    // Running locally
    const region = process.env.AWS_DEFAULT_REGION || process.env.AWS_REGION;
    const endpoint = process.env.GRAPL_AWS_ENDPOINT;
    const username = process.env.GRAPL_TEST_USER_NAME;
    const password = process.env.GRAPL_TEST_USER_PASSWORD_SECRET_ID;

    const credentials: aws_types.Credentials = {
      accessKeyId: process.env.GRAPL_AWS_ACCESS_KEY_ID ?? "",
      secretAccessKey: process.env.GRAPL_AWS_ACCESS_KEY_SECRET ?? "",
    };

    console.debug("Creating a local AWS client");

    return new clientType({
      endpoint: endpoint,
      region: region,
      credentials: credentials,
      username: username,
      password: password,
    });
  } else {
    // Running on AWS
    return new clientType({});
  }
}

export default getAwsClient;