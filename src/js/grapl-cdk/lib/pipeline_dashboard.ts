import * as cdk from '@aws-cdk/core';
import * as cloudwatch from '@aws-cdk/aws-cloudwatch';
import { Service } from './service';

function invocationsWidget(service: Service, isRetry?: boolean): cloudwatch.GraphWidget {
    const titleSuffix = isRetry ? " (retry)" : "";
    const handler = isRetry ? service.event_retry_handler : service.event_handler;
    return new cloudwatch.GraphWidget({
        title: `Invoke ${service.serviceName}${titleSuffix}`,
        left: [
            handler.metricInvocations({color: cloudwatch.Color.BLUE}),
            handler.metricErrors({color: cloudwatch.Color.RED}),
        ],
        width: 12,  // max of 24; we have 2 next to each other
        liveData: true,
    })
};

export class PipelineDashboard {
    constructor(
        scope: cdk.Construct,
        services: Service[],
    ) {
        const dashboard = new cloudwatch.Dashboard(
            scope, 
            'GraplPipelineDashboard', 
            {dashboardName: 'GraplPipelineDashboard'},
        );
        for (const service of services) {
            const invocations = invocationsWidget(service, false);
            const retryInvocations = invocationsWidget(service, true);
            dashboard.addWidgets(invocations, retryInvocations);
        }
    }
}