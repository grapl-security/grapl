import * as apigateway from "@aws-cdk/aws-apigateway";
import * as cdk from "@aws-cdk/core";
import * as cxapi from "@aws-cdk/cx-api";
import * as ec2 from "@aws-cdk/aws-ec2";
import * as s3 from "@aws-cdk/aws-s3";
import * as secretsmanager from "@aws-cdk/aws-secretsmanager";
import * as sns from "@aws-cdk/aws-sns";
import * as sqs from "@aws-cdk/aws-sqs";

import { Tags } from "@aws-cdk/core";

import { Service } from "./service";
import { UserAuthDb } from "./userauthdb";
import { EngagementNotebook } from "./engagement";
import { EngagementEdge } from "./engagement";
import { GraphQLEndpoint } from "./graphql";
import { OperationalAlarms, SecurityAlarms } from "./alarms";

import { Watchful } from "cdk-watchful";
import { SchemaDb } from "./schemadb";
import { PipelineDashboard } from "./pipeline_dashboard";
import { UxRouter } from "./ux_router";
import { GraplS3Bucket } from "./grapl_s3_bucket";
import { DGraphSwarmCluster } from "./services/dgraph_swarm_cluster";
import { ModelPluginDeployer } from "./services/model_plugin_deployer";
import { MetricForwarder } from "./services/metric_forwarder";
import { EngagementCreator } from "./services/engagement_creator";
import { AnalyzerExecutor } from "./services/analyzer_executor";
import { AnalyzerDispatch } from "./services/analyzer_dispatcher";
import { GraphMerger } from "./services/graph_merger";
import { NodeIdentifier } from "./services/node_identifier";
import { Provisioner } from "./services/provisioner";
import { SysmonGraphGenerator } from "./services/sysmon_graph_generator";
import { OSQueryGraphGenerator } from "./services/osquery_graph_generator";
import { LogLevels } from "../bin/deployment_parameters";

export interface GraplServiceProps {
    deploymentName: string;
    logLevels: LogLevels<string>;
    version: string;
    jwtSecret: secretsmanager.Secret;
    vpc: ec2.IVpc;
    dgraphSwarmCluster: DGraphSwarmCluster;
    userAuthTable: UserAuthDb;
    watchful?: Watchful;
    metricForwarder?: Service;
}

export interface GraplStackProps extends cdk.StackProps {
    stackName: string;
    logLevels: LogLevels<string>;
    version: string;
    watchfulEmail?: string;
    operationalAlarmsEmail: string;
    securityAlarmsEmail: string;
}

export class GraplCdkStack extends cdk.Stack {
    deploymentName: string;
    engagement_edge: EngagementEdge;
    graphql_endpoint: GraphQLEndpoint;
    ux_router: UxRouter;
    model_plugin_deployer: ModelPluginDeployer;
    edgeApiGateway: apigateway.RestApi;

    constructor(scope: cdk.Construct, id: string, props: GraplStackProps) {
        super(scope, id, props);

        const setFeatureFlags = () => {
            // ECS "Default Desired Count" is deprecated in favor of minScalingCapacity maxScalingCapacity
            this.node.setContext(cxapi.ECS_REMOVE_DEFAULT_DESIRED_COUNT, true);
        };
        setFeatureFlags();

        this.deploymentName = props.stackName;
        const deployment_name = this.deploymentName.toLowerCase();

        const edgeApi = new apigateway.RestApi(this, "EdgeApiGateway", {});
        edgeApi.addUsagePlan("EdgeApiGatewayUsagePlan", {
            quota: {
                limit: 1_000_000,
                period: apigateway.Period.DAY,
            },
            throttle: {
                // per minute
                rateLimit: 1200,
                burstLimit: 1200,
            },
        });

        this.edgeApiGateway = edgeApi;

        const grapl_vpc = new ec2.Vpc(this, this.deploymentName + "-VPC", {
            natGateways: 1,
            enableDnsHostnames: true,
            enableDnsSupport: true,
        });
        Tags.of(grapl_vpc).add(
            "name",
            `${this.deploymentName.toLowerCase()}-grapl-vpc`
        );

        const jwtSecret = new secretsmanager.Secret(this, "EdgeJwtSecret", {
            description:
                "The JWT secret that Grapl uses to authenticate its API",
            secretName: this.deploymentName + "-EdgeJwtSecret",
        });

        const user_auth_table = new UserAuthDb(this, "UserAuthTable", {
            table_name: this.deploymentName + "-user_auth_table",
        });

        const schema_table = new SchemaDb(this, "SchemaTable", {
            edges_table_name: this.deploymentName + "-grapl_schema_table",
            properties_table_name: `${this.deploymentName}-grapl_schema_properties_table`,
        });

        let watchful = undefined;
        if (props.watchfulEmail) {
            const alarmSqs = new sqs.Queue(this, "alarmSqs");
            const alarmSns = new sns.Topic(this, "alarmSns");

            watchful = new Watchful(this, id + "-Watchful", {
                alarmEmail: props.watchfulEmail,
                alarmSqs,
                alarmSns,
            });
        }

        const dgraphSwarmCluster = new DGraphSwarmCluster(this, "swarm", {
            deploymentName: this.deploymentName,
            vpc: grapl_vpc,
            version: props.version,
            watchful: watchful,
        });

        const graplProps: GraplServiceProps = {
            deploymentName: this.deploymentName,
            logLevels: props.logLevels,
            version: props.version,
            jwtSecret: jwtSecret,
            vpc: grapl_vpc,
            dgraphSwarmCluster: dgraphSwarmCluster,
            userAuthTable: user_auth_table,
            watchful: watchful,
        };

        const metric_forwarder = new MetricForwarder(this, "metric-forwarder", {
            ...graplProps,
        });
        // as we onboard more services to monitoring, add in ...enableMetricsProps
        const enableMetricsProps: Pick<GraplServiceProps, "metricForwarder"> = {
            metricForwarder: metric_forwarder.service,
        };

        const analyzers_bucket = new GraplS3Bucket(this, "AnalyzersBucket", {
            bucketName: deployment_name + "-analyzers-bucket",
            removalPolicy: cdk.RemovalPolicy.DESTROY,
            encryption: s3.BucketEncryption.KMS_MANAGED,
            blockPublicAccess: s3.BlockPublicAccess.BLOCK_ALL,
        });

        const engagements_created_topic = new sns.Topic(
            this,
            "EngagementsCreatedTopic",
            {
                topicName: this.deploymentName + "-engagements-created-topic",
            }
        );

        const engagement_creator = new EngagementCreator(
            this,
            "engagement-creator",
            {
                publishesTo: engagements_created_topic,
                ...graplProps,
                ...enableMetricsProps,
            }
        );

        const model_plugins_bucket = new GraplS3Bucket(
            this,
            "ModelPluginsBucket",
            {
                bucketName: deployment_name + "-model-plugins-bucket",
                removalPolicy: cdk.RemovalPolicy.DESTROY,
            }
        );

        this.model_plugin_deployer = new ModelPluginDeployer(
            this,
            "model-plugin-deployer",
            {
                modelPluginBucket: model_plugins_bucket,
                schemaTable: schema_table,
                edgeApi,
                ...graplProps,
            }
        );

        const analyzer_executor = new AnalyzerExecutor(
            this,
            "analyzer-executor",
            {
                writesTo: engagement_creator.bucket,
                readsAnalyzersFrom: analyzers_bucket,
                modelPluginsBucket: model_plugins_bucket,
                ...graplProps,
                ...enableMetricsProps,
            }
        );

        const analyzer_dispatch = new AnalyzerDispatch(
            this,
            "analyzer-dispatcher",
            {
                writesTo: analyzer_executor.sourceBucket,
                readsFrom: analyzers_bucket,
                ...graplProps,
                ...enableMetricsProps,
            }
        );

        const graph_merger = new GraphMerger(this, "graph-merger", {
            writesTo: analyzer_dispatch.bucket,
            schemaTable: schema_table,
            ...graplProps,
            ...enableMetricsProps,
        });

        const node_identifier = new NodeIdentifier(this, "node-identifier", {
            writesTo: graph_merger.bucket,
            ...graplProps,
            ...enableMetricsProps,
        });

        const sysmon_generator = new SysmonGraphGenerator(
            this,
            "sysmon-subgraph-generator",
            {
                writesTo: node_identifier.bucket,
                ...graplProps,
                ...enableMetricsProps,
            }
        );

        const osquery_generator = new OSQueryGraphGenerator(
            this,
            "osquery-subgraph-generator",
            {
                writesTo: node_identifier.bucket,
                ...graplProps,
                ...enableMetricsProps,
            }
        );

        const engagement_notebook = new EngagementNotebook(
            this,
            "engagements",
            {
                model_plugins_bucket,
                schema_db: schema_table,
                ...graplProps,
            }
        );

        this.engagement_edge = new EngagementEdge(this, "EngagementEdge", {
            ...graplProps,
            engagement_notebook: engagement_notebook,
            edgeApi,
        });

        const ux_bucket = new GraplS3Bucket(this, "EdgeBucket", {
            bucketName:
                graplProps.deploymentName.toLowerCase() +
                "-engagement-ux-bucket",
            publicReadAccess: false,
            websiteIndexDocument: "index.html",
            removalPolicy: cdk.RemovalPolicy.DESTROY,
        });

        this.ux_router = new UxRouter(this, "UxRouter", {
            ...graplProps,
            edgeApi,
        });

        this.graphql_endpoint = new GraphQLEndpoint(this, "GraphqlEndpoint", {
            ...graplProps,
            ux_bucket,
            edgeApi,
            schemaProperties: schema_table,
        });

        if (watchful) {
            const watchedOperations = [
                ...this.graphql_endpoint.apis,
                ...this.engagement_edge.apis,
                ...this.model_plugin_deployer.apis,
                ...this.ux_router.apis,
            ];

            watchful.watchApiGateway("EdgeApiGatewayIntegration", edgeApi, {
                serverErrorThreshold: 1, // any 5xx alerts
                cacheGraph: true,
                watchedOperations,
            });
        }

        new OperationalAlarms(this, "operational_alarms", {
            deployment_name: this.deploymentName,
            email: props.operationalAlarmsEmail,
        });

        new SecurityAlarms(this, "security_alarms", {
            deployment_name: this.deploymentName,
            email: props.securityAlarmsEmail,
        });

        new PipelineDashboard(this, "pipeline_dashboard", {
            namePrefix: this.deploymentName,
            services: [
                // Order here is important - the idea is that this dashboard will help Grapl operators
                // quickly determine which service in the pipeline is failing.
                sysmon_generator.service,
                osquery_generator.service,
                node_identifier.service,
                graph_merger.service,
                analyzer_dispatch.service,
                analyzer_executor.service,
                engagement_creator.service,
            ],
        });

        const provisioner = new Provisioner(this, "provisioner", {
            schemaDb: schema_table,
            ...graplProps,
        });
    }
}
