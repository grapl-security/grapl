import * as cdk from '@aws-cdk/core';
import * as cloudwatch from '@aws-cdk/aws-cloudwatch';
import { Service } from './service';
import {FargateService} from "./fargate_service";

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
            handler.metricInvocations({ color: cloudwatch.Color.BLUE }),
            handler.metricErrors({ color: cloudwatch.Color.RED }),
        ],
        width: 12, // max of 24; we have 2 next to each other
        liveData: true,
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
        width: 12, // max of 24; we have 2 next to each other
        liveData: true,
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
        for (const service of props.services) {
            if (service instanceof Service) {
                const invocations = lambdaInvocationsWidget(service, false);
                const retryInvocations = lambdaInvocationsWidget(service, true);
                dashboard.addWidgets(invocations, retryInvocations);
            } else if (service instanceof FargateService) {
                const invocations = fargateInvocationsWidget(service, false);
                const retryInvocations = fargateInvocationsWidget(service, true);
                dashboard.addWidgets(invocations, retryInvocations);
            } else {
                console.assert("service must be of type Service or FargateService", service, typeof service);
            }
        }
    }
}
