import * as cdk from "@aws-cdk/core";
import * as s3 from "@aws-cdk/aws-s3";
import * as sns from "@aws-cdk/aws-sns";
import * as ec2 from "@aws-cdk/aws-ec2";
import * as sqs from "@aws-cdk/aws-sqs";
import * as lambda from "@aws-cdk/aws-lambda";
import * as iam from "@aws-cdk/aws-iam";
import * as subscriptions from "@aws-cdk/aws-sns-subscriptions";

import { SqsEventSource } from '@aws-cdk/aws-lambda-event-sources';

class Queues {
    readonly queue: sqs.Queue;
    readonly retry_queue: sqs.Queue;

    constructor(scope: cdk.Construct, queue_name: string) {

        const dead_letter_queue = new sqs.Queue(scope, queue_name + '-dead-letter');

        this.retry_queue = new sqs.Queue(scope, queue_name + '-retry', {
            deadLetterQueue: { queue: dead_letter_queue, maxReceiveCount: 10 },
            visibilityTimeout: cdk.Duration.seconds(360)
        });

        this.queue = new sqs.Queue(scope, queue_name, {
            deadLetterQueue: { queue: this.retry_queue, maxReceiveCount: 5 },
            visibilityTimeout: cdk.Duration.seconds(180)
        });
    }
}

export interface ServiceProps {
    environment?: any,
    vpc?: ec2.IVpc,
    reads_from?: s3.IBucket,
    writes_to?: s3.IBucket,
    subscribes_to?: sns.ITopic,
    retry_code_name?: string,
    opt?: any
}

export class Service extends cdk.Construct {
    readonly event_handler: lambda.Function;
    readonly event_retry_handler: lambda.Function;
    readonly queues: Queues

    constructor(
        scope: cdk.Construct,
        name: string,
        props: ServiceProps
    ) {
        super(scope, name + "-Service");

        const environment = props.environment;
        let retry_code_name = props.retry_code_name;
        const opt = props.opt;

        const runtime = (opt && opt.runtime) ?
            opt.runtime : 
            {
                name: "provided",
                supportsInlineCode: true
            };
        const handler = (runtime === lambda.Runtime.PYTHON_3_7) ?
            `${name}.lambda_handler` :
            name;

        const queues = new Queues(this, name + '-queue');

        if (environment) {
            environment.QUEUE_URL = queues.queue.queueUrl;
            environment.RUST_BACKTRACE = "1";
        }

        const event_handler = new lambda.Function(
            this, name,
            {
                runtime: runtime,
                handler: handler,
                code: lambda.Code.asset(`./zips/${name}.zip`),
                vpc: props.vpc,
                environment: {
                    IS_RETRY: "False",
                    ...environment
                },
                timeout: cdk.Duration.seconds(180),
                memorySize: 256,
            });

        if (!retry_code_name) {
            retry_code_name = name
        }

        if (environment) {
            environment.QUEUE_URL = queues.retry_queue.queueUrl;
        }

        let event_retry_handler = new lambda.Function(
            this, name + '-retry-handler',
            {
                runtime: runtime,
                handler: handler,
                code: lambda.Code.asset(`./zips/${retry_code_name}.zip`),
                vpc: props.vpc,
                environment: {
                    IS_RETRY: "True",
                    ...environment
                },
                timeout: cdk.Duration.seconds(360),
                memorySize: 512,
            });

        event_handler.addEventSource(new SqsEventSource(queues.queue, { batchSize: 1 }));
        event_retry_handler.addEventSource(new SqsEventSource(queues.retry_queue, { batchSize: 1 }));

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
            this.addSubscription(
                scope,
                props.subscribes_to,
            );
        }

    }

    readsFrom(bucket: s3.IBucket, with_list?: Boolean) {
        let policy = new iam.PolicyStatement();
        policy.addActions('s3:GetObject', 's3:ActionGetBucket');

        if (with_list === true) {
            policy.addActions('s3:ListObjects')
        }

        policy.addResources(bucket.bucketArn);

        this.event_handler.addToRolePolicy(policy);
        this.event_retry_handler.addToRolePolicy(policy);

        // TODO: This is adding more permissions than necessary
        bucket.grantRead(this.event_handler);
        bucket.grantRead(this.event_retry_handler);
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

    addSubscription(
        scope: cdk.Construct,
        topic: sns.ITopic,
    ) {
        const subscription = new subscriptions.SqsSubscription(this.queues.queue)

        const config = subscription.bind(topic);

        new sns.Subscription(scope, 'Subscription', {
            topic: topic,
            endpoint: config.endpoint,
            filterPolicy: config.filterPolicy,
            protocol: config.protocol,
            rawMessageDelivery: true
        });
    }
}
