import * as cdk from '@aws-cdk/core';
import * as cloudwatch from '@aws-cdk/aws-cloudwatch';
import { Service } from './service';
import {FargateService} from "./fargate_service";
import * as sqs from '@aws-cdk/aws-sqs';

// like traffic lights, from best to worst
const GREEN_GRAPH = { color: cloudwatch.Color.GREEN };
const ORANGE_GRAPH = { color: cloudwatch.Color.ORANGE };
const RED_GRAPH = { color: cloudwatch.Color.RED };

// 24-width grid
const FULL_WIDTH = { width: 24 }; 
const HALF_WIDTH = { width: 12 };

function lambdaInvocationsWidget(
    service: Service,
    isRetry?: boolean
): cloudwatch.GraphWidget {
    const titleSuffix = isRetry ? ' (retry)' : '';
    const handler = isRetry
        ? service.event_retry_handler
        : service.event_handler;
    return new cloudwatch.GraphWidget({
        title: `Invoke ${service.serviceName}${titleSuffix}`,
        left: [
            handler.metricInvocations(ORANGE_GRAPH),
            handler.metricErrors(RED_GRAPH),
        ],
        liveData: true,
        ...HALF_WIDTH
    });
}

function fargateInvocationsWidget(
    service: FargateService,
    isRetry?: boolean
): cloudwatch.GraphWidget {
    const titleSuffix = isRetry ? ' (retry)' : '';
    const handler = isRetry
        ? service.service
        : service.retryService;

    return new cloudwatch.GraphWidget({
        title: `Invoke ${service.serviceName}${titleSuffix}`,
        left: [
            handler.service.metricCpuUtilization(),
            handler.service.metricMemoryUtilization(),
        ],
        liveData: true,
        ...HALF_WIDTH
    });
}

function fargateQueueWidget(
    service: FargateService,
    isRetry?: boolean
): cloudwatch.GraphWidget {
    let queues: sqs.Queue[] = [
    ]

    return new cloudwatch.GraphWidget({
        title: `Queues for ${service.serviceName}`,
        left: [
            // Num Messages Received is not necessarily the best 
            // metric to examine, but it's better than cpu/mem!
            service.queues.queue.metricNumberOfMessagesReceived({
                ...GREEN_GRAPH,
                label: "Queue",
            }),
            service.queues.retryQueue.metricNumberOfMessagesReceived({
                ...ORANGE_GRAPH,
                label: "Retry",
            }),
            // I'm using visible here since nobody is consuming from it.
            service.queues.deadLetterQueue.metricApproximateNumberOfMessagesVisible({
                ...RED_GRAPH,
                label: "Dead"
            }),
        ],
        liveData: true,
        ...FULL_WIDTH
    });
}


export class PipelineDashboardProps {
    readonly services: (Service | FargateService)[];
    readonly namePrefix: string;
}

export class PipelineDashboard extends cdk.Construct {
    constructor(
        scope: cdk.Construct,
        id: string,
        props: PipelineDashboardProps
    ) {
        super(scope, id);
        const dashboard = new cloudwatch.Dashboard(this, 'Dashboard', {
            dashboardName: props.namePrefix + '-PipelineDashboard',
        });
        // First, add metrics around queue health
        for (const service of props.services) {
            if (service instanceof Service) {
                const invocations = lambdaInvocationsWidget(service, false);
                const retryInvocations = lambdaInvocationsWidget(service, true);
                dashboard.addWidgets(invocations, retryInvocations);
            } else if (service instanceof FargateService) {
                const queueWidget = fargateQueueWidget(service);
                dashboard.addWidgets(queueWidget);
            } else {
                console.assert("service must be of type Service or FargateService", service, typeof service);
            }
        }
        
        dashboard.addWidgets(new cloudwatch.TextWidget({
            markdown: "Fargate service health",
            ...FULL_WIDTH,
        }));

        // Also, add metrics around service health for Fargate svcs
        for (const service of props.services) {
            if (service instanceof Service) {
                // do nothing
            } else if (service instanceof FargateService) {
                const invocations = fargateInvocationsWidget(service, false);
                const retryInvocations = fargateInvocationsWidget(service, true);
                dashboard.addWidgets(invocations, retryInvocations);
            } else {
                console.assert("service must be of type Service or FargateService, but was", typeof service);
            }
        }
    }
}
