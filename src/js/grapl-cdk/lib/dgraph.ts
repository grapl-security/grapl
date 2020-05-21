import * as cdk from "@aws-cdk/core";
import * as ec2 from "@aws-cdk/aws-ec2";
import * as ecs from "@aws-cdk/aws-ecs";
import * as servicediscovery from "@aws-cdk/aws-servicediscovery";

class Zero extends cdk.Construct {
    readonly name: string;

    constructor(
        scope: cdk.Construct,
        graph_name: string,
        id: string,
        cluster: ecs.Cluster,
        peer: string,
        idx: number
    ) {
        super(scope, graph_name + id);

        const zeroTask = new ecs.Ec2TaskDefinition(
            this, 
            id,
            {
                networkMode: ecs.NetworkMode.AWS_VPC,
            }
        );

        const command = ["dgraph", "zero", `--my=${id}.${graph_name}.grapl:5080`,
            "--replicas=3",
            `--idx=${idx}`,
            "--alsologtostderr"];

        if (peer) {
            command.push(`--peer=${peer}.${graph_name}.grapl:5080`);
        }

        const logDriver = new ecs.AwsLogDriver({
            streamPrefix: `ecs${graph_name + id}`,
        });

        zeroTask.addContainer(id + 'Container', {
            // --my is our own hostname (graph + id)
            // --peer is the other dgraph zero hostname
            image: ecs.ContainerImage.fromRegistry("dgraph/dgraph"),
            command,
            logging: logDriver,
            memoryReservationMiB: 1024,
        });

        const zeroService = new ecs.Ec2Service(this, id + 'Service', {
            cluster,  // Required
            taskDefinition: zeroTask,
            cloudMapOptions: {
                name: id,
                dnsRecordType: servicediscovery.DnsRecordType.A,
                dnsTtl: cdk.Duration.seconds(300),
            }
        });

        this.name = `${id}.${graph_name}.grapl`;

        zeroService.connections.allowFromAnyIpv4(
            ec2.Port.allTcp()
        );
    }
}

class Alpha extends cdk.Construct {
    readonly name: string;

    constructor(
        scope: cdk.Construct,
        graph_name: string,
        id: string,
        cluster: ecs.Cluster,
        zero: string
    ) {
        super(scope, graph_name + id);

        const alphaTask = new ecs.Ec2TaskDefinition(
            this,
            id,
            {
                networkMode: ecs.NetworkMode.AWS_VPC,
            }
        );

        const logDriver = new ecs.AwsLogDriver({
            streamPrefix: `ecs${graph_name + id}`,
        });

        alphaTask.addContainer(id + graph_name + 'Container', {
            image: ecs.ContainerImage.fromRegistry("dgraph/dgraph"),
            command: [
                "dgraph", "alpha", `--my=${id}.${graph_name}.grapl:7080`,
                "--lru_mb=1024", `--zero=${zero}.${graph_name}.grapl:5080`,
                "--alsologtostderr"
            ],
            logging: logDriver,
            memoryReservationMiB: 2048,
        });

        const alphaService = new ecs.Ec2Service(this, id + 'Service', {
            cluster,  // Required
            taskDefinition: alphaTask,
            cloudMapOptions: {
                name: id,
                dnsRecordType: servicediscovery.DnsRecordType.A,
                dnsTtl: cdk.Duration.seconds(300),
            }
        });

        this.name = `${id}.${graph_name}.grapl`;

        alphaService.connections.allowFromAnyIpv4(ec2.Port.allTcp());
    }
}

export class DGraphEcs extends cdk.Construct {
    readonly alphaNames: string[];

    constructor(
        scope: cdk.Construct,
        id: string,
        vpc: ec2.Vpc,
        zeroCount: number,
        alphaCount: number,
    ) {
        super(scope, id);

        const cluster = new ecs.Cluster(this, id + '-EcsCluster', {
            vpc: vpc
        });

        cluster.connections.allowInternally(ec2.Port.allTcp());

        const namespace = cluster.addDefaultCloudMapNamespace(
            {
                name: id + '.grapl',
                type: servicediscovery.NamespaceType.DNS_PRIVATE,
                vpc
            }
        );

        cluster.addCapacity(id + "ZeroGroupCapacity",
            {
                instanceType: new ec2.InstanceType("t3a.small"),
                minCapacity: zeroCount,
                maxCapacity: zeroCount,
            }
        );

        const zero0 = new Zero(
            this,
            id,
            'zero0',
            cluster,
            "",
            1
        );

        for (let i = 1; i < zeroCount; i++) {
            new Zero(
                this,
                id,
                `zero${i}`,
                cluster,
                'zero0',
                1 + i
            );
        }

        this.alphaNames = [];

        cluster.addCapacity(id + "AlphaGroupCapacity",
            {
                instanceType: new ec2.InstanceType("t3a.2xlarge"),
                minCapacity: alphaCount,
                maxCapacity: alphaCount,
            }
        );

        for (let i = 0; i < alphaCount; i++) {

            const alpha = new Alpha(
                this,
                id,
                `alpha${i}`, // increment for each alpha
                cluster,
                "zero0"
            );

            this.alphaNames.push(alpha.name);
        }
    }
}