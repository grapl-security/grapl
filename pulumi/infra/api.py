from typing import Optional

import pulumi_aws as aws
from infra.bucket import Bucket
from infra.config import LOCAL_GRAPL
from infra.dgraph_cluster import DgraphCluster
from infra.dynamodb import DynamoDB
from infra.engagement_edge import EngagementEdge
from infra.engagement_notebook import EngagementNotebook
from infra.graphql import GraphQL
from infra.lambda_ import Lambda
from infra.metric_forwarder import MetricForwarder
from infra.model_plugin_deployer import ModelPluginDeployer
from infra.network import Network
from infra.secret import JWTSecret
from infra.ux_router import UxRouter

import pulumi


class ProxyApiResource(pulumi.ComponentResource):
    """Helper class for setting up a lambda function as a proxy resource
    integrationon an APIGateway REST API.

        `path_part` is the path component in the API that the lambda
        function will be the proxy for. If `path_part` is not provided,
        the lambda will proxy the API's root resource. All `path_part`s
        for all `ProxyApiResource`s attached to a given REST API must be
        unique (including the empty `path_part` / root resource!).

        This class implicitly assumes that all `path_part`s are attached
        directly to the API's root resource; no provision is made for
        nested resources (e.g., a `path_part` containing `/` characters,
        such as "foo/bar/baz". This is only because we don't currently
        have a need for that.

        Lambda functions are assumed to be able to operate as an
        `AWS_PROXY` integration.

    """

    def __init__(
        self,
        api: aws.apigateway.RestApi,
        function: aws.lambda_.Function,
        path_part: Optional[str] = None,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:

        root_proxy = path_part is None
        path_part = "ROOT" if path_part is None else path_part

        super().__init__("grapl:ProxyAPIResource", path_part, None, opts)

        if not root_proxy:
            self.api_resource = aws.apigateway.Resource(
                f"{api._name}-{path_part}-api-resource",
                rest_api=api.id,
                parent_id=api.root_resource_id,
                path_part=path_part,
                opts=pulumi.ResourceOptions(parent=self),
            )

        self.proxy_resource = aws.apigateway.Resource(
            f"{api._name}-{path_part}-proxy-api-resource",
            rest_api=api.id,
            parent_id=api.root_resource_id if root_proxy else self.api_resource.id,
            path_part="{proxy+}",
            opts=pulumi.ResourceOptions(parent=self),
        )

        # This must be created before we can wire up a lambda
        # integration. "ANY" is an AWS-specific wildcard that means
        # any HTTP method can be used to invoke this particular
        # resource. The lambda function is responsible for dealing
        # with them.
        self.proxy_method = aws.apigateway.Method(
            f"{api._name}-{path_part}-proxy-api-ANY-method",
            authorization="NONE",
            http_method="ANY",
            resource_id=self.proxy_resource.id,
            rest_api=api.id,
            opts=pulumi.ResourceOptions(parent=self),
        )

        # Allow the lambda to be invoked with by API Gateway
        self.gateway_permission = aws.lambda_.Permission(
            f"{api._name}-{path_part}-invokes-{function._name}",
            principal=f"apigateway.amazonaws.com",
            action="lambda:InvokeFunction",
            function=function.arn,
            # The overall format here is:
            #
            #     arn:aws:execute-api:{region}:{account-id}:{api-id}/{stage-name}/{http-verb}/{resource-path-specifier}
            #
            # Currently we wildcard stage-name, HTTP verb, and proxied
            # path component.
            source_arn=pulumi.Output.concat(
                api.execution_arn,
                "/*/*/",  # stage name, HTTP verb
                "*" if root_proxy else f"{path_part}/*",
            ),
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.lambda_integration = aws.apigateway.Integration(
            f"{api._name}-{path_part}-{function._name}-integration",
            http_method=self.proxy_method.http_method,
            resource_id=self.proxy_resource.id,
            rest_api=api.id,
            type="AWS_PROXY",  # The only possibility for lambdas
            integration_http_method="POST",  # The only possibility for lambdas
            # Note: we're using the function's ARN in this URI, but we
            # could also use the function name, name+alias, or a
            # partial ARN:
            #
            # https://docs.aws.amazon.com/lambda/latest/dg/API_Invoke.html#API_Invoke_RequestSyntax
            uri=pulumi.Output.concat(
                "arn:aws:apigateway:",
                aws.get_region().name,
                ":lambda:path/2015-03-31/functions/",
                function.arn,
                "/invocations",
            ),
            opts=pulumi.ResourceOptions(
                parent=self, depends_on=[self.gateway_permission]
            ),
        )

        self.register_outputs({})


class Api(pulumi.ComponentResource):
    def __init__(
        self,
        network: Network,
        secret: JWTSecret,
        ux_bucket: Bucket,
        plugins_bucket: Bucket,
        db: DynamoDB,
        dgraph_cluster: DgraphCluster,
        forwarder: MetricForwarder,
        opts: Optional[pulumi.ResourceOptions] = None,
    ) -> None:
        name = "api-gateway"
        super().__init__("grapl:Api", name, None, opts)

        self.rest_api = aws.apigateway.RestApi(
            name, opts=pulumi.ResourceOptions(parent=self)
        )

        self.usage_plan = aws.apigateway.UsagePlan(
            f"{name}-usage-plan",
            quota_settings=aws.apigateway.UsagePlanQuotaSettingsArgs(
                limit=1_000_000,
                period="DAY",
            ),
            throttle_settings=aws.apigateway.UsagePlanThrottleSettingsArgs(
                burst_limit=1200, rate_limit=1200
            ),
            opts=pulumi.ResourceOptions(parent=self),
        )

        ########################################################################
        # TODO: Unsure if these two are actually needed? This mimics
        # the structure seen in the CDK-generated API gateway, but the
        # fact that I could call things from the Pulumi-generated API
        # gateway *without* these suggests they may be superfluous.
        #
        # Turns out it's also easier to add them here than as part of
        # the ProxyApiResource class
        #
        # I added them when trying to debug some localstack
        # shenanigans
        self.root_method = aws.apigateway.Method(
            f"{name}-ROOT-method",
            authorization="NONE",
            http_method="ANY",
            resource_id=self.rest_api.root_resource_id,
            rest_api=self.rest_api.id,
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.root_mock_integration = aws.apigateway.Integration(
            f"{name}-ROOT-MOCK-integration",
            http_method=self.root_method.http_method,
            resource_id=self.rest_api.root_resource_id,
            rest_api=self.rest_api.id,
            type="MOCK",
            opts=pulumi.ResourceOptions(parent=self),
        )

        ########################################################################

        # Our API currently consists of four services. Because
        # APIGateway deployments require all of their resources to be
        # in place first, and since all our resources are backed by
        # Lambda integrations, our deployment must formally depend on
        # each of those lambda integrations. The most straightforward
        # way to do that would seem to be making this API own
        # instances of each of them.

        self.ux_router = UxRouter(
            network=network,
            secret=secret,
            ux_bucket=ux_bucket,
            forwarder=forwarder,
        )

        # Sagemaker isn't currently supported in Localstack :/
        self.notebook = (
            EngagementNotebook(
                network=network,
                db=db,
                plugins_bucket=plugins_bucket,
                dgraph_cluster=dgraph_cluster,
                opts=pulumi.ResourceOptions(parent=self),
            )
            if not LOCAL_GRAPL
            else None
        )

        self.engagement_edge = EngagementEdge(
            network=network,
            secret=secret,
            db=db,
            notebook=self.notebook,
            forwarder=forwarder,
            dgraph_cluster=dgraph_cluster,
            opts=pulumi.ResourceOptions(parent=self),
        )

        self.model_plugin_deployer = ModelPluginDeployer(
            network=network,
            db=db,
            secret=secret,
            plugins_bucket=plugins_bucket,
            dgraph_cluster=dgraph_cluster,
            forwarder=forwarder,
        )

        # These don't work in LocalStack for some reason
        if not LOCAL_GRAPL:
            self.graphql_endpoint = GraphQL(
                network=network,
                secret=secret,
                ux_bucket=ux_bucket,
                db=db,
                dgraph_cluster=dgraph_cluster,
                forwarder=forwarder,
            )

        self.proxies = [
            self._add_proxy_resource_integration(self.ux_router.function),
            self._add_proxy_resource_integration(
                self.engagement_edge.function, path_part="auth"
            ),
            self._add_proxy_resource_integration(
                self.model_plugin_deployer.function,
                path_part="modelPluginDeployer",
            ),
        ]

        # These don't work in LocalStack for some reason
        if not LOCAL_GRAPL:
            self.proxies.extend(
                [
                    self._add_proxy_resource_integration(
                        self.graphql_endpoint.function, path_part="graphQlEndpoint"
                    ),
                ]
            )

        # This MUST be called after all integrations are registered in
        # self.proxies!
        self.stage = self._create_deployment()
        pulumi.export("prod-api-url", self.stage.invoke_url)
        pulumi.export("prod-api-id", self.stage.rest_api)

        self.register_outputs({})

    @property
    def invoke_url(self) -> pulumi.Output[str]:
        """ Returns the invocation URL of the "prod" stage of this API gateway. """
        return self.stage.invoke_url  # type: ignore[no-any-return]

    def _add_proxy_resource_integration(
        self,
        lambda_fn: Lambda,
        path_part: Optional[str] = None,
    ) -> ProxyApiResource:
        return ProxyApiResource(
            api=self.rest_api,
            function=lambda_fn.function,
            path_part=path_part,
            opts=pulumi.ResourceOptions(parent=self),
        )
        # self.triggers[lambda_fn.function._name] = lambda_fn.code_hash

    def _create_deployment(self) -> aws.apigateway.Stage:
        """ ONLY CALL THIS AFTER ALL RESOURCES ARE REGISTERED ON THE API GATEWAY! """
        deployment = aws.apigateway.Deployment(
            f"{self.rest_api._name}-deployment",
            rest_api=self.rest_api.id,
            # TODO: Still want to have a triggers function
            opts=pulumi.ResourceOptions(
                parent=self, depends_on=[p.lambda_integration for p in self.proxies]
            ),
        )

        stage = aws.apigateway.Stage(
            f"{self.rest_api._name}-prod-stage",
            deployment=deployment.id,
            rest_api=self.rest_api.id,
            stage_name="prod",
            opts=pulumi.ResourceOptions(parent=self),
        )

        return stage
