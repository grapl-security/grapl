import { Construct, CfnOutput } from '@aws-cdk/core';
import apigw = require('@aws-cdk/aws-apigateway');
import sns = require('@aws-cdk/aws-sns');
import sns_subscriptions = require('@aws-cdk/aws-sns-subscriptions');
import lambda = require('@aws-cdk/aws-lambda');
import cloudwatch_actions = require('@aws-cdk/aws-cloudwatch-actions');
import dynamodb = require('@aws-cdk/aws-dynamodb');
import sqs = require('@aws-cdk/aws-sqs');
import cloudwatch = require('@aws-cdk/aws-cloudwatch');
import { WatchDynamoTableOptions, WatchDynamoTable } from './dynamodb';
import { IWatchful, SectionOptions } from './api';
import { WatchLambdaFunctionOptions, WatchLambdaFunction } from './lambda';
import { WatchfulAspect, WatchfulAspectProps } from './aspect';
import { WatchApiGatewayOptions, WatchApiGateway } from './api-gateway';

export interface WatchfulProps {
  readonly alarmEmail?: string;
  readonly alarmSqs?: sqs.IQueue;
  readonly alarmSns?: sns.ITopic;
}

export class Watchful extends Construct implements IWatchful {
  private readonly dash: cloudwatch.Dashboard;
  private readonly alarmTopic?: sns.ITopic;

  constructor(scope: Construct, id: string, props: WatchfulProps = { }) {
    super(scope, id);

    if ((props.alarmEmail || props.alarmSqs) && !props.alarmSns) {
      this.alarmTopic = new sns.Topic(this, 'AlarmTopic', { displayName: 'Watchful Alarms' });
    }

    if (props.alarmSns) {
      this.alarmTopic = props.alarmSns;
    }

    if (props.alarmEmail && this.alarmTopic) {
      this.alarmTopic.addSubscription(
        new sns_subscriptions.EmailSubscription(props.alarmEmail)
      );
    }

    if (props.alarmSqs && this.alarmTopic) {
      this.alarmTopic.addSubscription(
        new sns_subscriptions.SqsSubscription(
          // sqs.Queue.fromQueueArn(this, 'AlarmQueue', props.alarmSqs)
          props.alarmSqs
        )
      );
    }

    this.dash = new cloudwatch.Dashboard(this, 'Dashboard');

    new CfnOutput(this, 'WatchfulDashboard', {
      value: linkForDashboard(this.dash)
    });
  }

  public addWidgets(...widgets: cloudwatch.IWidget[]) {
    this.dash.addWidgets(...widgets);
  }

  public addAlarm(alarm: cloudwatch.Alarm) {
    if (this.alarmTopic) {
      alarm.addAlarmAction(new cloudwatch_actions.SnsAction(this.alarmTopic));
    }
  }

  public addSection(title: string, options: SectionOptions = {}){
    const markdown = [
      `# ${title}`,
      (options.links || []).map(link => `[button:${link.title}](${link.url})`).join(' | ')
    ];

    this.addWidgets(new cloudwatch.TextWidget({ width: 24, markdown: markdown.join('\n') }));
  }

  public watchScope(scope: Construct, options?: WatchfulAspectProps) {
    const aspect = new WatchfulAspect(this, options);
    scope.node.applyAspect(aspect);
  }

  public watchDynamoTable(title: string, table: dynamodb.Table, options: WatchDynamoTableOptions = {}) {
    return new WatchDynamoTable(this, table.node.uniqueId, {
      title,
      watchful: this,
      table,
      ...options
    });
  }

  public watchApiGateway(title: string, restApi: apigw.RestApi, options: WatchApiGatewayOptions = {}) {
    return new WatchApiGateway(this, restApi.node.uniqueId, {
      title, watchful: this, restApi, ...options
    });
  }

  public watchLambdaFunction(title: string, fn: lambda.Function, options: WatchLambdaFunctionOptions = {}) {
    return new WatchLambdaFunction(this, fn.node.uniqueId, {
      title, watchful: this, fn, ...options
    });
  }
}

function linkForDashboard(dashboard: cloudwatch.Dashboard) {
  const cfnDashboard = dashboard.node.defaultChild as cloudwatch.CfnDashboard;
  return `https://console.aws.amazon.com/cloudwatch/home?region=${dashboard.stack.region}#dashboards:name=${cfnDashboard.ref}`;
}
