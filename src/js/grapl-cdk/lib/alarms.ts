import * as cdk from '@aws-cdk/core';
import * as sns from '@aws-cdk/aws-sns';
import * as subs from '@aws-cdk/aws-sns-subscriptions';
import * as cloudwatch from '@aws-cdk/aws-cloudwatch';
import * as cloudwatch_actions from '@aws-cdk/aws-cloudwatch-actions';

class AlarmSink {
    readonly topic: sns.Topic;

    constructor(scope: cdk.Construct, id: string, emailAddress: string) {
        this.topic = new sns.Topic(scope, id);
        this.topic.addSubscription(new subs.EmailSubscription(emailAddress));
    }
}

class RiskNodeAlarm {
    constructor(
        scope: cdk.Construct,
        alarm_sink: AlarmSink,
    ) {
        const metric = new cloudwatch.Metric({
            namespace: 'engagement-creator',
            metricName: 'risk_node',
        });
        const alarm = metric.createAlarm(
            scope,
            "risk_node_alarm",
            {
                alarmName: "risk_node_alarm",
                // if it happens once in a given hour, send an alert.
                threshold: 1,
                // Default period is 5 minutes
                evaluationPeriods: 15,
            }
        );
        alarm.addAlarmAction(
            new cloudwatch_actions.SnsAction(alarm_sink.topic)
        );
    }
}

export class OperationalAlarms {
    // Alarms meant for the operator of the Grapl stack.
    // That is to say: Grapl Inc (in the Grapl Cloud case), and VeryCool Corp (in the on-prem case).
    constructor(
        scope: cdk.Construct,
    ) {
        // We probably want this email to be configurable, and sent to our operators - not necessarily
        const alarm_sink = new AlarmSink(scope, "operational_alarm_sink", "operational-alarms@graplsecurity.com");
    }
}


export class SecurityAlarms {
    // Alarms meant for the consumer of the Grapl stack.
    constructor(
        scope: cdk.Construct,
    ) {
        // We probably want this email to be configurable, and sent to our customers - not us.
        const alarm_sink = new AlarmSink(scope, "security_alarm_sink", "security-alarms@graplsecurity.com");
        const risk_node_alarm = new RiskNodeAlarm(scope, alarm_sink);
    }
}