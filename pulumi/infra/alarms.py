from typing import Optional

import pulumi_aws as aws
from infra.config import STACK_NAME, get_grapl_ops_alarms_email

import pulumi


class AlarmSink(pulumi.ComponentResource):
    """
    A place for an Alarm to go.
    """

    def __init__(
        self,
        name: str,
        *,
        topic_name_suffix: str,
        email: str,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:AlarmSink", name=name, props=None, opts=opts)
        child_opts = pulumi.ResourceOptions(parent=self)

        topic_name = f"{STACK_NAME}-{topic_name_suffix}"
        self.topic = aws.sns.Topic(
            f"topic-{name}",
            name=topic_name,
            opts=child_opts,
        )

        # When you send something to this SNS, emit an email
        aws.sns.TopicSubscription(
            f"topic-subscription-{name}",
            topic=self.topic.arn,
            protocol="email",
            endpoint=email,
            opts=child_opts,
        )


class OpsAlarms(pulumi.ComponentResource):
    """
    Alarms meant for the operator of the Grapl stack.
    That is to say: Grapl Inc (in the Grapl Cloud case), or VeryCool Corp (in the on-prem case).
    """

    def __init__(
        self,
        name: str,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:OpsAlarms", name=name, props=None, opts=opts)

        child_opts = pulumi.ResourceOptions(parent=self)
        self.alarm_sink = AlarmSink(
            "alarm-sink",
            topic_name_suffix="operational-alarms-sink",
            email=get_grapl_ops_alarms_email(),
            opts=child_opts,
        )
        # Usually, we'd define metrics that would send a Cloudwatch Action
        # to that Alarm Sink here, in Pulumi.
        # However, the only place that actually sends metrics to this sink is
        # the `graplctl`-provisioned EC2 instances, where it looks it up by topic name.
