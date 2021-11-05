import json
from typing import List, Optional

import pulumi_aws as aws
from typing_extensions import Final

import pulumi

API_GATEWAY_LOGS_JSON_FORMAT: Final[str] = json.dumps(
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

# > The $default route catches requests that don't explicitly match other
# > routes in your API.
# https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-develop-routes.html
#
# Since we want to send *all* routes to `web-ui`, this captures all routes.
_API_GATEWAY_DEFAULT_ROUTE_KEY: Final[str] = "$default"

# > You can create a $default stage that is served from the base of your API's
# > URL—for example, https://{api_id}.execute-api.{region}.amazonaws.com/.
# > You use this URL to invoke an API stage.
# https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-stages.html
#
# Naming it $default means that the stage is served from
# https://api-id.execute-api.us-east-1.amazonaws.com/
# instead of something like
# https://api-id.execute-api.us-east-1.amazonaws.com/stage-1a2b3c4
# this makes it much easier to reference stuff in e.g. `<domain>/static/`
_API_GATEWAY_DEFAULT_STAGE_NAME: Final[str] = "$default"


class ApiGateway(pulumi.ComponentResource):
    def __init__(
        self,
        name: str,
        # These 2 properties are exported from nomad_agents_stack and refer
        # to the application load balancer on the autoscaling group.
        nomad_agents_alb_security_group: pulumi.Input[str],
        nomad_agents_alb_listener_arn: pulumi.Input[str],
        # This property is exported from networking_stack and defines the
        # subnet the Nomad Agents Cluster runs on
        nomad_agents_private_subnet_ids: pulumi.Input[List[str]],
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        """
        Quick diatribe on how this is all hooked up:
        Incoming request
        --> API Gateway Route (snapshotted as a Stage)
        --> API Gateway "integration"
        --> VPCLink
          (everything beyond here is defined in `platform-infrastructure`)
        --> Nomad Agents ALB Listener port
        --> Nomad Agents ALB
        --> ALB Target Group's port
        --> a real Nomad Agent instance's port
          (everything beyond here is defined in Nomad job files)
        --> Nomad ingress gateway (`grapl-ingress.nomad`)
        --> `web-ui` service
        """
        super().__init__("grapl:ApiGateway", name, None, opts)

        api = aws.apigatewayv2.Api(
            f"{name}-api",
            protocol_type="HTTP",
            opts=pulumi.ResourceOptions(parent=self),
        )

        # Use apigwv2's VpcLink for http integrations.
        vpc_link = aws.apigatewayv2.VpcLink(
            "{name}-vpc-link-to-nomad-ingress",
            security_group_ids=[nomad_agents_alb_security_group],
            subnet_ids=nomad_agents_private_subnet_ids,
            opts=pulumi.ResourceOptions(parent=api),
        )

        integration = aws.apigatewayv2.Integration(
            f"{name}-vpc-link-integration",
            api_id=api.id,
            connection_id=vpc_link.id,
            connection_type="VPC_LINK",
            description="Integrate the VPC Link to Nomad with API Gateway",
            integration_method="ANY",
            integration_type="HTTP_PROXY",
            integration_uri=nomad_agents_alb_listener_arn,
            opts=pulumi.ResourceOptions(parent=api),
        )

        route = aws.apigatewayv2.Route(
            f"{name}-vpc-link-route",
            api_id=api.id,
            route_key=_API_GATEWAY_DEFAULT_ROUTE_KEY,
            # For info on this weird transformation, see
            # https://www.pulumi.com/registry/packages/aws/api-docs/apigatewayv2/route/#target_python
            target=integration.id.apply(lambda id: f"integrations/{id}"),
            opts=pulumi.ResourceOptions(parent=api),
        )

        stage_logs = aws.cloudwatch.LogGroup(
            f"{name}-log-group",
            name=f"/{name}-apigw-stage-logs",
            retention_in_days=7,
            opts=pulumi.ResourceOptions(parent=api),
        )

        self.stage = aws.apigatewayv2.Stage(
            f"{name}-vpc-link-stage",
            name=_API_GATEWAY_DEFAULT_STAGE_NAME,
            api_id=api.id,
            auto_deploy=True,
            access_log_settings=aws.apigatewayv2.StageAccessLogSettingsArgs(
                destination_arn=stage_logs.arn,
                format=API_GATEWAY_LOGS_JSON_FORMAT,
            ),
            opts=pulumi.ResourceOptions(parent=api),
        )

        self.register_outputs({})
