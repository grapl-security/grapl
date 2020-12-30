import * as cdk from '@aws-cdk/core';
import * as s3 from '@aws-cdk/aws-s3';
import * as iam from '@aws-cdk/aws-iam';
import * as sns from '@aws-cdk/aws-sns';
import * as subscriptions from '@aws-cdk/aws-sns-subscriptions';

import * as ec2 from "@aws-cdk/aws-ec2";
import * as ecs from "@aws-cdk/aws-ecs";
import * as sqs from "@aws-cdk/aws-sqs";
import * as ecs_patterns from "@aws-cdk/aws-ecs-patterns";
import {ContainerImage} from "@aws-cdk/aws-ecs";
import {GraplServiceProps} from "./grapl-cdk-stack";
import {Watchful} from "cdk-watchful";

export class Queues {
    readonly queue: sqs.Queue;
    readonly retry_queue: sqs.Queue;
    readonly deadLetterQueue: sqs.Queue;

    constructor(scope: cdk.Construct, queue_name: string) {
        this.deadLetterQueue = new sqs.Queue(scope, 'DeadLetterQueue', {
            queueName: queue_name + '-dead-letter-queue',
        });

        this.retry_queue = new sqs.Queue(scope, 'RetryQueue', {
            queueName: queue_name + '-retry-queue',
            deadLetterQueue: { queue: this.deadLetterQueue, maxReceiveCount: 3 },
            visibilityTimeout: cdk.Duration.seconds(360),
        });

        this.queue = new sqs.Queue(scope, 'Queue', {
            queueName: queue_name + '-queue',
            deadLetterQueue: { queue: this.retry_queue, maxReceiveCount: 3 },
            visibilityTimeout: cdk.Duration.seconds(180),
        });
    }
}


export interface FargateServiceProps {
    prefix: string;
    version: string;
    readsFrom: s3.IBucket;
    subscribesTo: sns.ITopic;
    serviceImage: ContainerImage;
    retryServiceImage?: ContainerImage | undefined;
    vpc: ec2.IVpc;
    environment: {
        [key: string]: string;
    };
    writesTo?: s3.IBucket | undefined;
    command?: string[] | undefined;
    retryCommand?: string[] | undefined;
    watchful?: Watchful | undefined;
}

export class FargateService {
    readonly queues: Queues;
    readonly serviceName: string;
    readonly service: ecs_patterns.QueueProcessingFargateService;
    readonly retryService: ecs_patterns.QueueProcessingFargateService;

    constructor(scope: cdk.Construct, serviceName: string, props: FargateServiceProps) {
        const cluster = new ecs.Cluster(scope, `${serviceName}Cluster`, {
            vpc: props.vpc,
        });

        const queues = new Queues(scope, `${serviceName}`);

        this.queues = queues;
        this.serviceName = serviceName;

        const defaultEnv: { [key: string]: string; } = {
            "BUCKET_PREFIX": props.prefix,
            "DEAD_LETTER_QUEUE_URL": this.queues.deadLetterQueue.queueUrl,
            "SOURCE_BUCKET_NAME": props.readsFrom.bucketName,
            "GRAPL_LOG_LEVEL": "DEBUG",
            "RUST_LOG": "warn,main=debug,sqs-executor=info",
        };

        const optionalEnv: { [key: string]: string;} = {};
        if (props.writesTo) {
            optionalEnv["DEST_BUCKET_NAME"] = props.writesTo.bucketName;
        }

        // Create a load-balanced Fargate service and make it public
        this.service = new ecs_patterns.QueueProcessingFargateService(
            scope,
            `${props.prefix}-${serviceName}Service`, {
            cluster,
            command: props.command,
            enableLogging: true,
            environment: {
                "QUEUE_URL": this.queues.queue.queueUrl,
                "SOURCE_QUEUE_URL": this.queues.queue.queueUrl,
                ...optionalEnv,
                ...defaultEnv,
                ...props.environment
            },
            image: props.serviceImage,
            queue: queues.queue,
            serviceName,
            cpu: 256,
            memoryLimitMiB: 512,
            desiredTaskCount: 1,
        });

        this.retryService = new ecs_patterns.QueueProcessingFargateService(
            scope,
            `${props.prefix}-${serviceName}RetryService`, {
                cluster,
                command: props.retryCommand || props.command,
                enableLogging: true,
                environment: {
                    "QUEUE_URL": this.queues.retry_queue.queueUrl,
                    "SOURCE_QUEUE_URL": this.queues.retry_queue.queueUrl,
                    ...optionalEnv,
                    ...defaultEnv,
                    ...props.environment
                },
                image: props.retryServiceImage || props.serviceImage,
                queue: queues.retry_queue,
                serviceName: serviceName+"-retry-handler",
                cpu: 256,
                memoryLimitMiB: 512,
                desiredTaskCount: 1,
            });


        // todo: we need permissions to publish to the dead letter queue
        this.queues.deadLetterQueue.grantSendMessages(this.service.taskDefinition.taskRole);
        this.queues.deadLetterQueue.grantSendMessages(this.retryService.taskDefinition.taskRole);

        if (props.readsFrom) {
            this.readsFrom(props.readsFrom);
        }

        if (props.writesTo) {
            this.writesToBucket(props.writesTo);
        }

        if (props.subscribesTo) {
            this.addSubscription(scope, props.subscribesTo);
        }
    }

    readsFrom(bucket: s3.IBucket, with_list?: Boolean) {
        let policy = new iam.PolicyStatement({
            effect: iam.Effect.ALLOW,
            actions: ['s3:GetObject'],
            resources: [bucket.bucketArn + '/*'],
        });

        if (with_list === true) {
            policy.addResources(bucket.bucketArn);
            policy.addActions('s3:ListBucket');
        }

        this.service.service.taskDefinition.addToTaskRolePolicy(policy);
    }

    publishesToTopic(publishes_to: sns.ITopic) {
        const topicPolicy = new iam.PolicyStatement({
            effect: iam.Effect.ALLOW,
            actions: ['sns:CreateTopic', 'sns:Publish'],
            resources: [publishes_to.topicArn],
        });

        this.service.service.taskDefinition.addToTaskRolePolicy(topicPolicy);
    }

    writesToBucket(publishes_to: s3.IBucket) {
        publishes_to.grantWrite(this.service.service.taskDefinition.taskRole);
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


