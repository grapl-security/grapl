import * as cdk from '@aws-cdk/core';
import * as ec2 from "@aws-cdk/aws-ec2";
import * as ecs from "@aws-cdk/aws-ecs";
import * as sqs from "@aws-cdk/aws-sqs";
import * as ecs_patterns from "@aws-cdk/aws-ecs-patterns";
import {ContainerImage} from "@aws-cdk/aws-ecs";

export class Queues {
    readonly queue: sqs.Queue;
    readonly retry_queue: sqs.Queue;

    constructor(scope: cdk.Construct, queue_name: string) {
        const dead_letter_queue = new sqs.Queue(scope, 'DeadLetterQueue', {
            queueName: queue_name + '-dead-letter-queue',
        });

        this.retry_queue = new sqs.Queue(scope, 'RetryQueue', {
            queueName: queue_name + '-retry-queue',
            deadLetterQueue: { queue: dead_letter_queue, maxReceiveCount: 3 },
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
    serviceName: string;
    serviceImage: ContainerImage;
    vpc: ec2.Vpc;
    queues: Queues;
}

export class FargateService {
    // readonly queues: Queues;
    // readonly serviceName: string;

    constructor(scope: cdk.Construct, name: string, props: FargateServiceProps) {
        const cluster = new ecs.Cluster(scope, `${props.serviceName}Cluster`, {
            vpc: props.vpc,
        });

        // Create a load-balanced Fargate service and make it public
        new ecs_patterns.QueueProcessingFargateService(
            scope,
            `${props.serviceName}Service`, {
            cluster,
            enableLogging: true,
            environment: {

            },
            image: props.serviceImage,
            queue: props.queues.queue,
            serviceName: props.serviceName,
            vpc: props.vpc
        });
    }
}


