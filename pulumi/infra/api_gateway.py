import json
from typing import List, Optional

import pulumi_aws as aws

import pulumi

API_GATEWAY_LOGS_JSON_FORMAT = json.dumps(
    {
        # This is verbatim what you get from clicking the JSON button at
        # https://console.aws.amazon.com/apigateway/main/monitor/logging/edit
        "requestId": "$context.requestId",
        "ip": "$context.identity.sourceIp",
        "requestTime": "$context.requestTime",
        "httpMethod": "$context.httpMethod",
        "routeKey": "$context.routeKey",
        "status": "$context.status",
        "protocol": "$context.protocol",
        "responseLength": "$context.responseLength",
        # And some extra- integration-specific ones.
        "integrationError": "$context.integration.error",
        "integrationStatus": "$context.integration.status",
        "responseType": "$context.error.responseType",
        "message": "$context.error.message",
    }
)


class ApiGateway(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        nomad_agents_alb_security_group: pulumi.Input[str],
        nomad_agents_alb_listener_arn: pulumi.Input[str],
        private_subnet_ids: pulumi.Input[List[str]],
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        super().__init__("grapl:ApiGateway", name, None, opts)

        api = aws.apigatewayv2.Api(
            "api",
            protocol_type="HTTP",
            opts=pulumi.ResourceOptions(parent=self),
        )

        # Use apigwv2's VpcLink for http integrations.
        link = aws.apigatewayv2.VpcLink(
            "vpc_link_to_ec2",
            security_group_ids=[nomad_agents_alb_security_group],
            subnet_ids=private_subnet_ids,
            opts=pulumi.ResourceOptions(parent=api),
        )

        integration = aws.apigatewayv2.Integration(
            "integration",
            api_id=api.id,
            connection_id=link.id,
            connection_type="VPC_LINK",
            integration_method="ANY",
            integration_type="HTTP_PROXY",
            integration_uri=nomad_agents_alb_listener_arn,
            request_parameters={
                # AWS calls this "Parameter Mapping."
                # This one strips the `/stage-xyz` from incoming requests.
                "overwrite:path": "$request.path",
            },
            opts=pulumi.ResourceOptions(parent=api),
        )

        default_route = aws.apigatewayv2.Route(
            "default_route",
            api_id=api.id,
            route_key="$default",
            target=integration.id.apply(lambda id: f"integrations/{id}"),
            opts=pulumi.ResourceOptions(parent=api),
        )

        stage_logs = aws.cloudwatch.LogGroup(
            f"{name}-log-group",
            name=f"/{name}-apigw_stage_logs",
            retention_in_days=7,
            opts=pulumi.ResourceOptions(parent=api),
        )

        self.stage = aws.apigatewayv2.Stage(
            "stage",
            api_id=api.id,
            auto_deploy=True,
            access_log_settings=aws.apigatewayv2.StageAccessLogSettingsArgs(
                destination_arn=stage_logs.arn,
                format=API_GATEWAY_LOGS_JSON_FORMAT,
            ),
            opts=pulumi.ResourceOptions(parent=api),
        )

        self.register_outputs({})
