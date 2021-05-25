import json

import pulumi_aws as aws
from infra.config import DEPLOYMENT_NAME, GLOBAL_LAMBDA_ZIP_TAG, configurable_envvars
from infra.dgraph_cluster import DgraphCluster
from infra.emitter import EventEmitter
from infra.lambda_ import code_path_for
from infra.metric_forwarder import MetricForwarder
from infra.network import Network
from infra.service import Service

import pulumi


class EngagementCreator(Service):
    def __init__(
        self,
        input_emitter: EventEmitter,
        network: Network,
        forwarder: MetricForwarder,
        dgraph_cluster: DgraphCluster,
    ) -> None:

        name = "engagement-creator"
        super().__init__(
            name,
            forwarder=forwarder,
            lambda_description=GLOBAL_LAMBDA_ZIP_TAG,
            lambda_handler_fn="lambdex_handler.handler",
            lambda_code_path=code_path_for("engagement-creator"),
            network=network,
            env={
                **configurable_envvars(name, ["GRAPL_LOG_LEVEL"]),
                "MG_ALPHAS": dgraph_cluster.alpha_host_port,
            },
        )

        self.queue.subscribe_to_emitter(input_emitter)
        input_emitter.grant_read_to(self.role)

        physical_topic_name = f"{DEPLOYMENT_NAME}-engagements-created-topic"
        self.created_topic = aws.sns.Topic(
            "engagements-created-topic",
            name=physical_topic_name,
            opts=pulumi.ResourceOptions(parent=self),
        )

        publish_to_topic_policy = self.created_topic.arn.apply(
            lambda topic_arn: json.dumps(
                {
                    "Version": "2012-10-17",
                    "Statement": [
                        {
                            "Effect": "Allow",
                            # TODO: Do we need CreateTopic? In any
                            # event, this is what was in our CDK code
                            "Action": ["sns:CreateTopic", "sns:Publish"],
                            "Resource": topic_arn,
                        }
                    ],
                }
            )
        )

        self.topic_policy_attachment = aws.iam.RolePolicy(
            f"{name}-publishes-to-topic",
            name=f"{DEPLOYMENT_NAME}-{name}-publishes-to-topic",
            role=self.role.name,
            policy=publish_to_topic_policy,
            opts=pulumi.ResourceOptions(parent=self.role),
        )

        for handler in self.handlers:
            dgraph_cluster.allow_connections_from(handler.function.security_group)

        self.register_outputs({})
