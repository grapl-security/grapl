import * as cdk from "@aws-cdk/core";
import * as ec2 from "@aws-cdk/aws-ec2";
import * as ecs from "@aws-cdk/aws-ecs";
import * as servicediscovery from "@aws-cdk/aws-servicediscovery";
import * as service_common from "../service_common";
import * as logs from "@aws-cdk/aws-logs";

import {GraplServiceProps} from "../grapl-cdk-stack";
import {ContainerImage} from "@aws-cdk/aws-ecs";
import {RUST_DOCKERFILE, SRC_DIR} from "../dockerfile_paths";
import {IConnectable} from "@aws-cdk/aws-ec2";
import {SchemaDb} from "../schemadb";

interface GraphMutationServiceProps extends GraplServiceProps {
    graphMutationServiceRustBuild?: string | undefined;
    grpcPort?: number | undefined;
    schemaDb: SchemaDb;
}

export class GraphMutationServiceStack extends cdk.Construct {
    readonly serviceName: string;
    readonly fargateService: ecs.FargateService;
    readonly grpcPort: number;

    constructor(scope: cdk.Construct, id: string, props: GraphMutationServiceProps) {
        super(scope, id);
        this.grpcPort = props.grpcPort || 5500;
        const cluster = new ecs.Cluster(this, "GraphMutationServiceCluster", {
            vpc: props.vpc,
            defaultCloudMapNamespace: {
                name: "graph-mutation-service.grapl",
                type: servicediscovery.NamespaceType.DNS_PRIVATE,
                vpc: props.vpc
            }
        });

        const taskDefinition = new ecs.FargateTaskDefinition(this, "graph_mutation_service_task_def", {
            cpu: 256,
            memoryLimitMiB: 512,
        });

        const logGroup = new logs.LogGroup(scope, "default", {
            logGroupName: `grapl/${this.serviceName}/default`,
            removalPolicy: cdk.RemovalPolicy.DESTROY,
            retention: service_common.LOG_RETENTION,
        });

        const logDriver = new ecs.AwsLogDriver({
            streamPrefix: `grapl-graph-mutation-service`,
            logGroup,
        });

        const serviceContainer = taskDefinition.addContainer("grapl-graph-mutation-service-container", {
            image: getContainer(props),
            memoryLimitMiB: 128,
            logging: logDriver,
            environment: makeEnvironment(props),
        });
        serviceContainer.addPortMappings({
            containerPort: this.grpcPort,
        });

        this.fargateService = new ecs.FargateService(this, "graph_mutation_fargate_service", {
            cluster: cluster,
            desiredCount: 1,
            assignPublicIp: false,
            taskDefinition,
            cloudMapOptions: {
                dnsRecordType: servicediscovery.DnsRecordType.A,
                dnsTtl: cdk.Duration.seconds(10),
                failureThreshold: 2,
                name: "graph-mutation-service"
            }
        });

        props.schemaDb.allowReadFromRole(this.fargateService.taskDefinition.taskRole);
        props.dgraphSwarmCluster.allowConnectionsFrom(this.fargateService.cluster.connections);
    }

    grantAccess = (access_from: IConnectable): void => {
        access_from
            .connections
            .allowTo(this.fargateService, ec2.Port.tcp(this.grpcPort), "Allow grpc to graph mutation service");
    }
}

const getContainer = (props: GraphMutationServiceProps): ContainerImage => {
    return ContainerImage.fromAsset(SRC_DIR, {
        target: "graph-mutation-service-deploy",
        buildArgs: {
            // TODO: We should be defaulting to *release*, not debug
            RUST_BUILD: props.graphMutationServiceRustBuild || "debug",
        },
        file: RUST_DOCKERFILE,
    })
}

type GraphMutationServiceEnvironment = {
    GRAPL_SCHEMA_TABLE: string;
    IS_LOCAL: string;
    MG_ALPHAS: string;
}

interface EnvironmentDependencies extends GraphMutationServiceProps {
    // TODO: Currently the graph-mutation-service actually doens't do any redis-based work
    // redisCluster: RedisCluster,
}

const makeEnvironment = (props: EnvironmentDependencies): GraphMutationServiceEnvironment => {
    return {
        GRAPL_SCHEMA_TABLE: props.schemaDb.schema_table.tableName,
        IS_LOCAL: "False",
        MG_ALPHAS: props.dgraphSwarmCluster.alphaHostPort(),
    }
}
