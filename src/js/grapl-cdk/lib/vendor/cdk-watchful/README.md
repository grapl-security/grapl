# cdk-watchful

[![CircleCI](https://circleci.com/gh/eladb/cdk-watchful.svg?style=svg)](https://circleci.com/gh/eladb/cdk-watchful)
[![python](https://img.shields.io/badge/jsii-python-blueviolet.svg)](https://pypi.org/project/cdk-watchful/)
[![typescript](https://img.shields.io/badge/jsii-typescript-blueviolet.svg)](https://www.npmjs.com/package/cdk-watchful)

> Watching your CDK back since 2019

Watchful is an [AWS CDK](https://github.com/awslabs/aws-cdk) construct library that makes it easy
to monitor CDK apps.

**TypeScript:**

```ts
import { Watchful } from 'cdk-watchful'

const wf = new Watchful(this, 'watchful');
wf.watchDynamoTable('My Cute Little Table', myTable);
wf.watchLambdaFunction('My Function', myFunction);
wf.watchApiGateway('My REST API', myRestApi);
```

**Python:**

```python
from cdk_watchful import Watchful

wf = Watchful(self, 'watchful')
wf.watch_dynamo_table('My Cute Little Table', my_table)
wf.watch_lambda_function('My Function', my_function)
wf.watch_api_gateway('My REST API', my_rest_api)
```

And...

![](https://raw.githubusercontent.com/eladb/cdk-watchful/master/example/sample.png)

## Install

TypeScript/JavaScript:

```console
$ npm install cdk-watchful
```

Python:

```console
$ pip install cdk-watchful
```

## Initialize

To get started, just define a `Watchful` construct in your CDK app (code is in
TypeScript, but python will work too). You can initialize using an email address, SQS arn or both:

**TypeScript:**

```ts
import { Watchful } from 'cdk-watchful'
import sns = require('@aws-cdk/aws-sns');
import sqs = require('@aws-cdk/aws-sqs');

const alarmSqs = sqs.Queue.fromQueueArn(this, 'AlarmQueue', 'arn:aws:sqs:us-east-1:444455556666:alarm-queue')
const alarmSns = sns.Topic.fromTopicArn(this, 'AlarmTopic', 'arn:aws:sns:us-east-2:444455556666:MyTopic');

const wf = new Watchful(this, 'watchful', {
  alarmEmail: 'your@email.com',
  alarmSqs,
  alarmSns,
});
```

**Python:**

```python
from cdk_watchful import Watchful

alarm_sqs = sqs.Queue.from_queue_arn(self, 'AlarmQueue', 'arn:aws:sqs:us-east-1:444455556666:alarm-queue')
alarm_sns = sns.Topic.from_topic_arn(self, 'AlarmTopic', 'arn:aws:sns:us-east-2:444455556666:MyTopic')

wf = Watchful(
  self,
  'watchful',
  alarm_email='your@amil.com',
  alarm_sqs=alarm_sqs,
  alarm_sns=alarm_sns
```

## Add Resources

Watchful manages a central dashboard and configures default alarming for:

- Amazon DynamoDB: `watchful.watchDynamoTable`
- AWS Lambda: `watchful.watchLambdaFunction`
- Amazon API Gateway: `watchful.watchApiGateway`
- [Request yours](https://github.com/eladb/cdk-watchful/issues/new)

## Watching Scopes

Watchful can also watch complete CDK construct scopes. It will automatically
discover all watchable resources within that scope (recursively), add them
to your dashboard and configure alarms for them.

**TypeScript:**

```ts
wf.watchScope(storageLayer);
```

**Python:**

```python
wf.watch_scope(storage_layer)
```

## Example

See a more complete [example](https://github.com/eladb/cdk-watchful/blob/master/example/index.ts).

## License

[Apache 2.0](https://github.com/eladb/cdk-watchful/blob/master/LICENSE)

