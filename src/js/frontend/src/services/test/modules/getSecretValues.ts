import { SecretsManagerClient, GetSecretValueCommand } from "@aws-sdk/client-secrets-manager"; // ES Modules import
import getAwsClient from "./envHelpers";

const getSecrets = async () => {
  const client = new SecretsManagerClient(getAwsClient);
  const command = new GetSecretValueCommand({ SecretId: 'GRAPL_TEST_USER_PASSWORD_SECRET_ID' });
  const response = await client.send(command);
  console.log("secret manager response", response)
  return response
}

export default getSecrets();