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
        super(scope, id);

        const zeroTask = new ecs.Ec2TaskDefinition(
            this,
            'TaskDef',
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
            image: ecs.ContainerImage.fromRegistry("dgraph/dgraph:v1.2.2"),
            command,
            logging: logDriver,
            memoryReservationMiB: 1024,
        });

        const zeroService = new ecs.Ec2Service(this, 'Ec2Service', {
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
        super(scope, id);

        const alphaTask = new ecs.Ec2TaskDefinition(
            this,
            'TaskDef',
            {
                networkMode: ecs.NetworkMode.AWS_VPC,
            }
        );

        const logDriver = new ecs.AwsLogDriver({
            streamPrefix: `ecs${graph_name + id}`,
        });

        alphaTask.addContainer(id + graph_name + 'Container', {
            image: ecs.ContainerImage.fromRegistry("dgraph/dgraph:v1.2.2"),
            command: [
                "dgraph", "alpha", `--my=${id}.${graph_name}.grapl:7080`,
                "--lru_mb=1024", `--zero=${zero}.${graph_name}.grapl:5080`,
                "--alsologtostderr"
            ],
            logging: logDriver,
            memoryReservationMiB: 2048,
        });

        const alphaService = new ecs.Ec2Service(this, 'Ec2Service', {
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

export interface DGraphEcsProps {
    prefix: string,
    vpc: ec2.Vpc,
    alphaCount: number,
    alphaPort: number
    zeroCount: number,
}

export class DGraphEcs extends cdk.Construct {
    readonly alphas: [string, number][];

    constructor(
        scope: cdk.Construct,
        id: string,
        props: DGraphEcsProps,
    ) {
        super(scope, id);

        const cluster = new ecs.Cluster(this, 'EcsCluster', {
            clusterName: `${props.prefix}-${id}-EcsCluster`,
            vpc: props.vpc
        });

        cluster.connections.allowInternally(ec2.Port.allTcp());

        cluster.addDefaultCloudMapNamespace(
            {
                name: id + '.grapl',
                type: servicediscovery.NamespaceType.DNS_PRIVATE,
                vpc: props.vpc,
            }
        );

        cluster.addCapacity('ZeroGroupCapacity',
            {
                instanceType: new ec2.InstanceType("t3a.small"),
                minCapacity: props.zeroCount,
                maxCapacity: props.zeroCount,
            }
        );

        new Zero(
            this,
            id,
            'zero0',
            cluster,
            "",
            1
        );

        for (let i = 1; i < props.zeroCount; i++) {
            new Zero(
                this,
                id,
                `zero${i}`,
                cluster,
                'zero0',
                1 + i
            );
        }

        this.alphas = [];

        cluster.addCapacity('AlphaGroupCapacity',
            {
                instanceType: new ec2.InstanceType("t3a.2xlarge"),
                minCapacity: props.alphaCount,
                maxCapacity: props.alphaCount,
            }
        );

        for (let i = 0; i < props.alphaCount; i++) {

            const alpha = new Alpha(
                this,
                id,
                `alpha${i}`, // increment for each alpha
                cluster,
                "zero0"
            );

            this.alphas.push([alpha.name, props.alphaPort]);
        }
    };

    alphaHostPorts(): string[] {
        let names: string[] = []
        this.alphas.forEach(function (value) {
            let [host, port] = value;
            names.push(`${host}:${port}`);
        });

        return names;
    }
}
