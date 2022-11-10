import { SecretsManagerClient, GetSecretValueCommand } from "@aws-sdk/client-secrets-manager"; // ES Modules import
import getAwsClient from "./envHelpers";

const getValueFromAWSSecretsManager = async (secretId: string) => {
  const client = getAwsClient(SecretsManagerClient);
  const command = new GetSecretValueCommand({ SecretId: secretId });
  const response = await client.send(command);
  return response.SecretString;
};

export default getValueFromAWSSecretsManager;
