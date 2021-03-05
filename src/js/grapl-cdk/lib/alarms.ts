import * as cdk from '@aws-cdk/core';
import * as sns from '@aws-cdk/aws-sns';
import * as subs from '@aws-cdk/aws-sns-subscriptions';
import * as cloudwatch from '@aws-cdk/aws-cloudwatch';
import * as cloudwatch_actions from '@aws-cdk/aws-cloudwatch-actions';

/**
 * Our alarms setup will likely change heavily, and we may even move off of Cloudwatch in the near future.
 * 
 * WARNING to anybody adding alarms: Cloudwatch Alarms doesn't allow alarms based on SEARCH(), meaning,
 * you must define *concrete metrics* to alarm on. 
 * 
 * As an example:
 * If you had a metric:
 * "I spotted a dog", dimensions: {"breed": "shar-pei", age: "puppy"}
 * "I spotted a dog", dimensions: {"breed": "beagle", age: "adult"}
 * 
 * You would NOT BE ABLE to create an alarm based on a generic "new dog spotted". 
 * Instead, you'd have to create an alarm that manually specifies every single combination of dimensions; 
 * in this case an alarm of, manually specified,
 *  {i saw a sharpei puppy + i saw a sharpei adult + i saw a beagle puppy + i saw a beagle adult + ...}
 * (and also, this maxes out at 10 metrics)
 * 
 * As such: I think should probably just emit a metric - just for alarms - that has no dimensions; as well as a separate
 * metric that perhaps provides that extra context.
 */

class AlarmSinkProps {
    topic_name: string;
    email: string;
    deployment_name: string;
}

class AlarmSink extends cdk.Construct {
    readonly topic: sns.Topic;
    readonly cloudwatch_action: cloudwatch_actions.SnsAction;

    constructor(scope: cdk.Construct, id: string, props: AlarmSinkProps) {
        super(scope, id);

        const topic_name = `${props.deployment_name}-${props.topic_name}`;
        this.topic = new sns.Topic(scope, "topic", {
            topicName: topic_name
        });
        this.topic.addSubscription(new subs.EmailSubscription(props.email));
        this.cloudwatch_action = new cloudwatch_actions.SnsAction(this.topic)
    }
}

interface RiskNodeAlarmProps {
    deployment_name: string;
    alarm_sink: AlarmSink;
}

class RiskNodeAlarm extends cdk.Construct {
    constructor(
        scope: cdk.Construct,
        id: string,
        props: RiskNodeAlarmProps,
    ) {
        super(scope, id);
        const metric = new cloudwatch.Metric({
            namespace: 'engagement-creator',
            metricName: 'risk_node',
            dimensions: {},
        });
        const alarm = metric.createAlarm(
            this,
            "alarm",
            {
                alarmName: props.deployment_name + " - Risk node alarm",
                // TODO: Add some verbiage to the alarm description on how to actually look at what's causing the alarm.
                alarmDescription: undefined,
                threshold: 1,
                evaluationPeriods: 1,
                treatMissingData: cloudwatch.TreatMissingData.NOT_BREACHING,
            }
        );
        alarm.addAlarmAction(props.alarm_sink.cloudwatch_action);
    }
}

export interface OperationalAlarmsProps {
    deployment_name: string;
    email: string;
}

export class OperationalAlarms extends cdk.Construct {
    // Alarms meant for the operator of the Grapl stack.
    // That is to say: Grapl Inc (in the Grapl Cloud case), or VeryCool Corp (in the on-prem case).
    constructor(
        scope: cdk.Construct,
        id: string,
        props: OperationalAlarmsProps,
    ) {
        super(scope, id);
        const alarm_sink_props: AlarmSinkProps = {
            topic_name: "operational-alarms-sink",
            ...props
        }
        const alarm_sink = new AlarmSink(this, "alarm_sink", alarm_sink_props);
    }
}

export interface SecurityAlarmsProps {
    deployment_name: string;
    email: string;
}

export class SecurityAlarms extends cdk.Construct {
    // Alarms meant for the consumer of the Grapl stack - for example, alarms triggered by analyzers.
    constructor(
        scope: cdk.Construct,
        id: string,
        props: SecurityAlarmsProps,
    ) {
        super(scope, id);
        const alarm_sink_props: AlarmSinkProps = {
            topic_name: "security-alarms-sink",
            ...props
        }
        const alarm_sink = new AlarmSink(this, "alarm_sink", alarm_sink_props);
        const risk_node_alarm = new RiskNodeAlarm(this, "risk_node_alarm", {
            deployment_name: props.deployment_name,
            alarm_sink: alarm_sink
        });
    }
}