import * as cdk from "@aws-cdk/core";
import * as s3 from "@aws-cdk/aws-s3";
import * as iam from "@aws-cdk/aws-iam";
import * as sns from "@aws-cdk/aws-sns";
import * as subscriptions from "@aws-cdk/aws-sns-subscriptions";

import * as logs from "@aws-cdk/aws-logs";
import * as lambda from "@aws-cdk/aws-lambda";
import * as ec2 from "@aws-cdk/aws-ec2";
import * as ecs from "@aws-cdk/aws-ecs";
import * as sqs from "@aws-cdk/aws-sqs";
import * as ecs_patterns from "@aws-cdk/aws-ecs-patterns";
import { ContainerImage, AwsLogDriver } from "@aws-cdk/aws-ecs";
import { Watchful } from "cdk-watchful";
import { EventEmitter } from "./event_emitters";
import { LambdaDestination } from "@aws-cdk/aws-logs-destinations";
import { Service } from "./service";
import * as service_common from "./service_common";
import { QueueProcessingFargateServiceProps } from "@aws-cdk/aws-ecs-patterns";

export class Queues {
    readonly queue: sqs.Queue;
    readonly retryQueue: sqs.Queue;
    readonly deadLetterQueue: sqs.Queue;

    constructor(scope: cdk.Construct, queueName: string) {
        this.deadLetterQueue = new sqs.Queue(scope, "DeadLetterQueue", {
            queueName: queueName + "-dead-letter-queue",
        });

        this.retryQueue = new sqs.Queue(scope, "RetryQueue", {
            queueName: queueName + "-retry-queue",
            deadLetterQueue: {
                queue: this.deadLetterQueue,
                maxReceiveCount: 3,
            },
            visibilityTimeout: cdk.Duration.seconds(360),
        });

        this.queue = new sqs.Queue(scope, "Queue", {
            queueName: queueName + "-queue",
            deadLetterQueue: { queue: this.retryQueue, maxReceiveCount: 3 },
            visibilityTimeout: cdk.Duration.seconds(180),
        });
    }
}

export interface FargateServiceProps {
    deploymentName: string;
    version: string;
    // We read events from this
    eventEmitter: EventEmitter;
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
    metric_forwarder?: Service;
}

interface DefaultAndRetry<T> {
    readonly default: T;
    readonly retry: T;
}

function getAutoscalingProps({
    minTasks,
    maxTasks,
}: {
    minTasks: number;
    maxTasks: number;
}): Partial<QueueProcessingFargateServiceProps> {
    return {
        // Fargate autoscaling groups can adjust their scaling based on any metric, like:
        // CPU usage, approximate queue messages, etc.
        // The QueueProcessingFargateService pattern does it based on both of those metrics!
        // https://github.com/aws/aws-cdk/blob/7966f8d48c4bff26beb22856d289f9d0c7e7081d/packages/%40aws-cdk/aws-ecs-patterns/lib/base/queue-processing-service-base.ts#L331

        // Due to a bug, we have to also specify desiredCapacity: https://github.com/aws/aws-cdk/issues/14336
        desiredTaskCount: minTasks,
        minScalingCapacity: minTasks,
        maxScalingCapacity: maxTasks,

        // These numbers are based on ApproximateNumberOfMessagesVisible in the input queue
        // This hasn't been calibrated at all
        scalingSteps: [
            { upper: 0, change: -1 }, // Scale down to minimum if no messages
            { lower: 1, change: +1 }, // Scale up a bit if _any_ messages (particularly important when minTasks == 0)
            { lower: 100, change: +2 },
            { lower: 500, change: +5 },
        ],
    };
}

export class FargateService {
    readonly queues: Queues;
    readonly serviceName: string;
    readonly service: ecs_patterns.QueueProcessingFargateService;
    readonly retryService: ecs_patterns.QueueProcessingFargateService;
    readonly logGroups: DefaultAndRetry<logs.LogGroup>;

    constructor(
        scope: cdk.Construct,
        serviceName: string,
        props: FargateServiceProps
    ) {
        this.serviceName = `${props.deploymentName}-${serviceName}`;
        const cluster = new ecs.Cluster(scope, `${this.serviceName}-cluster`, {
            vpc: props.vpc,
            clusterName: `${this.serviceName}-cluster`,
        });
        const readsFrom = props.eventEmitter.bucket;
        const subscribesTo = props.eventEmitter.topic;

        const queues = new Queues(scope, this.serviceName);

        this.queues = queues;

        const defaultEnv: { [key: string]: string } = {
            DEPLOYMENT_NAME: props.deploymentName,
            DEAD_LETTER_QUEUE_URL: this.queues.deadLetterQueue.queueUrl,
            GRAPL_LOG_LEVEL: "DEBUG",
            RUST_LOG: "warn,main=debug,sqs-executor=info",
            RETRY_QUEUE_URL: this.queues.retryQueue.queueUrl,
        };

        const optionalEnv: { [key: string]: string } = {};
        if (props.writesTo) {
            optionalEnv["DEST_BUCKET_NAME"] = props.writesTo.bucketName;
        }

        this.logGroups = {
            default: new logs.LogGroup(scope, "default", {
                logGroupName: `grapl/${this.serviceName}/default`,
                removalPolicy: cdk.RemovalPolicy.DESTROY,
                retention: service_common.LOG_RETENTION,
            }),
            retry: new logs.LogGroup(scope, "retry", {
                logGroupName: `grapl/${this.serviceName}/retry`,
                removalPolicy: cdk.RemovalPolicy.DESTROY,
                retention: service_common.LOG_RETENTION,
            }),
        };

        const defaultAutoscaling = getAutoscalingProps({
            minTasks: 1,
            maxTasks: 4,
        });

        // Scaling to zero causes some latency issues. We've decided to enable
        // it for retry services. Basically a tradeoff between cost and how long it takes to process.
        // https://grapl-internal.slack.com/archives/C018YCSN0B0/p1620156010045800?thread_ts=1620154431.041100&cid=C018YCSN0B0
        const retryAutoscaling = getAutoscalingProps({
            minTasks: 0,
            maxTasks: 4,
        });

        // Create a load-balanced Fargate service and make it public
        this.service = new ecs_patterns.QueueProcessingFargateService(
            scope,
            `${this.serviceName}-service`,
            {
                cluster,
                serviceName: `${this.serviceName}-handler`,
                family: `${this.serviceName}-task`,
                command: props.command,
                enableLogging: true,
                environment: {
                    QUEUE_URL: this.queues.queue.queueUrl,
                    SOURCE_QUEUE_URL: this.queues.queue.queueUrl,
                    ...optionalEnv,
                    ...defaultEnv,
                    ...props.environment,
                },
                image: props.serviceImage,
                queue: queues.queue,
                cpu: 256,
                memoryLimitMiB: 512,
                logDriver: new AwsLogDriver({
                    streamPrefix: "logs",
                    logGroup: this.logGroups.default,
                }),
                ...defaultAutoscaling,
            }
        );

        this.retryService = new ecs_patterns.QueueProcessingFargateService(
            scope,
            `${this.serviceName}-retry-service`,
            {
                cluster,
                serviceName: `${this.serviceName}-retry-handler`,
                family: `${this.serviceName}-retry-task`,
                command: props.retryCommand || props.command,
                enableLogging: true,
                environment: {
                    QUEUE_URL: this.queues.retryQueue.queueUrl,
                    SOURCE_QUEUE_URL: this.queues.retryQueue.queueUrl,
                    ...optionalEnv,
                    ...defaultEnv,
                    ...props.environment,
                },
                image: props.retryServiceImage || props.serviceImage,
                queue: queues.retryQueue,
                cpu: 256,
                memoryLimitMiB: 512,
                logDriver: new AwsLogDriver({
                    streamPrefix: "logs",
                    logGroup: this.logGroups.retry,
                }),
                ...retryAutoscaling,
            }
        );

        for (const q of [
            this.queues.queue,
            this.queues.retryQueue,
            this.queues.deadLetterQueue,
        ]) {
            q.grantConsumeMessages(this.service.taskDefinition.taskRole);
            q.grantConsumeMessages(this.retryService.taskDefinition.taskRole);
            q.grantSendMessages(this.service.taskDefinition.taskRole);
            q.grantSendMessages(this.retryService.taskDefinition.taskRole);
        }

        if (readsFrom) {
            this.readsFromBucket(readsFrom);
        }

        if (props.writesTo) {
            this.writesToBucket(props.writesTo);
        }

        if (subscribesTo) {
            this.addSubscription(scope, subscribesTo);
        }

        if (props.metric_forwarder) {
            const forwarder_lambda = props.metric_forwarder.event_handler;
            this.forwardMetricsLogs(forwarder_lambda);
        }
    }

    readsFromBucket(bucket: s3.IBucket, with_list?: Boolean) {
        let policy = new iam.PolicyStatement({
            effect: iam.Effect.ALLOW,
            actions: ["s3:GetObject"],
            resources: [bucket.bucketArn + "/*"],
        });

        if (with_list === true) {
            policy.addResources(bucket.bucketArn);
            policy.addActions("s3:ListBucket");
        }

        this.service.service.taskDefinition.addToTaskRolePolicy(policy);
    }

    publishesToTopic(publishes_to: sns.ITopic) {
        const topicPolicy = new iam.PolicyStatement({
            effect: iam.Effect.ALLOW,
            actions: ["sns:CreateTopic", "sns:Publish"],
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

        new sns.Subscription(scope, "Subscription", {
            topic: topic,
            endpoint: config.endpoint,
            filterPolicy: config.filterPolicy,
            protocol: config.protocol,
            rawMessageDelivery: true,
        });
    }

    forwardMetricsLogs(toLambdaFn: lambda.IFunction) {
        this.logGroups.default.addSubscriptionFilter(
            `send_metrics_to_lambda_${this.serviceName}`,
            {
                destination: new LambdaDestination(toLambdaFn),
                filterPattern: logs.FilterPattern.literal("MONITORING"),
            }
        );
        this.logGroups.retry.addSubscriptionFilter(
            `send_metrics_to_lambda_${this.serviceName}_retry`,
            {
                destination: new LambdaDestination(toLambdaFn),
                filterPattern: logs.FilterPattern.literal("MONITORING"),
            }
        );
    }

    grantListQueues() {
        // Some of our code, locally, tests for SQS availability with a `list_queues` call.
        // In the interest of unified local and prod code, we can grant this permission.
        const policy = new iam.PolicyStatement({
            effect: iam.Effect.ALLOW,
            actions: ["sqs:ListQueues"],
            resources: ["*"],
        });

        this.service.taskDefinition.addToTaskRolePolicy(policy);
        this.retryService.taskDefinition.addToTaskRolePolicy(policy);
    }
}
