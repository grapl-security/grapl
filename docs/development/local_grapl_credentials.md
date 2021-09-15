# Local Grapl credentials

(This article should expire by about Oct or Nov 2021, when we'll have more
robust user management.)

Your username is:

```
local-grapl-grapl-test-user
```

You can retrieve your password with:

```
awslocal secretsmanager get-secret-value --secret-id local-grapl-TestUserPassword | jq -r ".SecretString"
```

To auth against `grapl-web-ui`:

```
YOUR_NOMAD_IP="1.2.3.4"  # fill this in - you can find it at localhost:4646 UI

PASSWORD=$(awslocal secretsmanager get-secret-value --secret-id local-grapl-TestUserPassword | jq -r ".SecretString")
curl -i --location --request POST "http://${YOUR_NOMAD_IP}:8666/auth/login" \
--header 'content-type: application/json' \
--data @- << REQUEST_BODY
{
    "username": "local-grapl-grapl-test-user",
    "password": "${PASSWORD}"
}
REQUEST_BODY
```
