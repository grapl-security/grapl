import { SecretsManagerClient, GetSecretValueCommand } from "@aws-sdk/client-secrets-manager"; // ES Modules import
import getAwsClient from "./envHelpers";

const getSecrets = async () => {
  const client = getAwsClient(SecretsManagerClient);
  const command = new GetSecretValueCommand({ SecretId: "local-grapl-TestUserPassword" });
  const response = await client.send(command);
  console.log("secret manager response", response);
  return response;
};

export default getSecrets();
