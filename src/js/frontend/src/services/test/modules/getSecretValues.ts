import { SecretsManagerClient, GetSecretValueCommand } from "@aws-sdk/client-secrets-manager"; // ES Modules import
import getAwsClient from "./envHelpers";

const getTestPasswordFromAWSSecretsManager = async () => {
  const client = getAwsClient(SecretsManagerClient);
  const command = new GetSecretValueCommand({ SecretId: "local-grapl-TestUserPassword" });
  const response = await client.send(command);
  return response.SecretString;
};

export default getTestPasswordFromAWSSecretsManager();
