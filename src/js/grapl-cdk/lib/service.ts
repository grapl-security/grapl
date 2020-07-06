import * as cdk from '@aws-cdk/core';
import * as s3 from '@aws-cdk/aws-s3';
import * as sns from '@aws-cdk/aws-sns';
import * as ec2 from '@aws-cdk/aws-ec2';
import * as sqs from '@aws-cdk/aws-sqs';
import * as lambda from '@aws-cdk/aws-lambda';
import * as iam from '@aws-cdk/aws-iam';
import * as subscriptions from '@aws-cdk/aws-sns-subscriptions';

import { SqsEventSource } from '@aws-cdk/aws-lambda-event-sources';

import { Watchful } from './vendor/cdk-watchful/lib/watchful';

class Queues {
    readonly queue: sqs.Queue;
    readonly retry_queue: sqs.Queue;

    constructor(scope: cdk.Construct, queue_name: string) {
        const dead_letter_queue = new sqs.Queue(scope, 'DeadLetterQueue', {
            queueName: queue_name + '-dead-letter-queue',
        });

        this.retry_queue = new sqs.Queue(scope, 'RetryQueue', {
            queueName: queue_name + '-retry-queue',
            deadLetterQueue: { queue: dead_letter_queue, maxReceiveCount: 10 },
            visibilityTimeout: cdk.Duration.seconds(360),
        });

        this.queue = new sqs.Queue(scope, 'Queue', {
            queueName: queue_name + '-queue',
            deadLetterQueue: { queue: this.retry_queue, maxReceiveCount: 5 },
            visibilityTimeout: cdk.Duration.seconds(180),
        });
    }
}

export interface ServiceProps {
    version: string;
    prefix: string;
    environment?: any;
    vpc?: ec2.IVpc;
    reads_from?: s3.IBucket;
    writes_to?: s3.IBucket;
    subscribes_to?: sns.ITopic;
    retry_code_name?: string;
    opt?: any;
    watchful?: Watchful;
}

export class Service {
    readonly event_handler: lambda.IFunction;
    readonly event_retry_handler: lambda.Function;
    readonly queues: Queues;

    constructor(scope: cdk.Construct, name: string, props: ServiceProps) {
        const serviceName = `${props.prefix}-${name}`;
        const environment = props.environment;
        let retry_code_name = props.retry_code_name;
        const opt = props.opt;

        const runtime =
            opt && opt.runtime
                ? opt.runtime
                : {
                      name: 'provided',
                      supportsInlineCode: true,
                  };

        const handler =
            runtime === lambda.Runtime.PYTHON_3_7
                ? `${name}.lambda_handler`
                : name;

        const queues = new Queues(scope, serviceName.toLowerCase());

        if (environment) {
            environment.SOURCE_QUEUE_URL = queues.queue.queueUrl;
            environment.RUST_BACKTRACE = '1';
        }

        const event_handler = new lambda.Function(scope, 'Handler', {
            runtime: runtime,
            handler: handler,
            functionName: serviceName + '-Handler',
            code: lambda.Code.asset(`./zips/${name}-${props.version}.zip`),
            vpc: props.vpc,
            environment: {
                IS_RETRY: 'False',
                ...environment,
            },
            timeout: cdk.Duration.seconds(180),
            memorySize: 256,
            description: props.version,
        });
        event_handler.currentVersion.addAlias('live');

        if (props.watchful) {
            props.watchful.watchLambdaFunction(
                name + '-Handler',
                event_handler
            );
        }

        if (!retry_code_name) {
            retry_code_name = name;
        }

        if (environment) {
            environment.SOURCE_QUEUE_URL = queues.retry_queue.queueUrl;
        }

        let event_retry_handler = new lambda.Function(scope, 'RetryHandler', {
            runtime: runtime,
            handler: handler,
            functionName: serviceName + '-RetryHandler',
            code: lambda.Code.asset(
                `./zips/${retry_code_name}-${props.version}.zip`
            ),
            vpc: props.vpc,
            environment: {
                IS_RETRY: 'True',
                ...environment,
            },
            timeout: cdk.Duration.seconds(360),
            memorySize: 512,
            description: props.version,
        });
        event_retry_handler.currentVersion.addAlias('live');

        if (props.watchful) {
            props.watchful.watchLambdaFunction(
                name + '-RetryHandler',
                event_retry_handler
            );
        }

        event_handler.addEventSource(
            new SqsEventSource(queues.queue, { batchSize: 10 })
        );
        event_retry_handler.addEventSource(
            new SqsEventSource(queues.retry_queue, { batchSize: 10 })
        );

        queues.queue.grantConsumeMessages(event_handler);
        queues.retry_queue.grantConsumeMessages(event_retry_handler);

        this.queues = queues;
        this.event_handler = event_handler;
        this.event_retry_handler = event_retry_handler;

        if (props.reads_from) {
            this.readsFrom(props.reads_from);
        }

        if (props.writes_to) {
            this.publishesToBucket(props.writes_to);
        }

        if (props.subscribes_to) {
            this.addSubscription(scope, props.subscribes_to);
        }
    }

    readsFrom(bucket: s3.IBucket, with_list?: Boolean) {
        let policy = new iam.PolicyStatement({
            sid: 'Allow GetObject',
            effect: iam.Effect.ALLOW,
            actions: ['s3:GetObject'],
            resources: [bucket.bucketArn + '/*']
        });

        if (with_list === true) {
            policy.addActions('s3:ListObjects');
        }

        this.event_handler.addToRolePolicy(policy);
        this.event_retry_handler.addToRolePolicy(policy);
    }

    publishesToTopic(publishes_to: sns.ITopic) {
        const topicPolicy = new iam.PolicyStatement();

        topicPolicy.addActions('sns:CreateTopic');
        topicPolicy.addResources(publishes_to.topicArn);

        this.event_handler.addToRolePolicy(topicPolicy);

        this.event_retry_handler.addToRolePolicy(topicPolicy);

        publishes_to.grantPublish(this.event_handler);
        publishes_to.grantPublish(this.event_retry_handler);
    }

    publishesToBucket(publishes_to: s3.IBucket) {
        publishes_to.grantWrite(this.event_handler);
        publishes_to.grantWrite(this.event_retry_handler);
    }

    addSubscription(scope: cdk.Construct, topic: sns.ITopic) {
        const subscription = new subscriptions.SqsSubscription(
            this.queues.queue
        );

        const config = subscription.bind(topic);

        new sns.Subscription(scope, 'Subscription', {
            topic: topic,
            endpoint: config.endpoint,
            filterPolicy: config.filterPolicy,
            protocol: config.protocol,
            rawMessageDelivery: true,
        });
    }
}
