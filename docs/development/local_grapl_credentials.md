# Local Grapl credentials
(This article should expire by about Oct or Nov 2021, when we'll have more robust user management.)

Your username is:
```
local-grapl-grapl-test-user
```

You can retrieve your password with:
```
awslocal secretsmanager get-secret-value --secret-id local-grapl-TestUserPassword | jq -r ".SecretString"
```
