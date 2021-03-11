from __future__ import annotations

import pathlib
import time
import uuid
from typing import TYPE_CHECKING, Callable, Dict, Iterator, List, Optional

import boto3
import click
from graplctl import __version__, aws_cdk_ops, common, dgraph_ops, docker_swarm_ops
from graplctl.common import GraplctlState, pass_graplctl_state
from graplctl.queues.commands import queues

if TYPE_CHECKING:
    from mypy_boto3_ssm import SSMClient

Tag = common.Tag
Ec2Instance = common.Ec2Instance


def _ticker(n: int) -> Iterator[None]:
    for _ in range(n):
        time.sleep(1)
        yield None


#
# main entrypoint for grapctl
#


@click.group()
@click.version_option(version=__version__)
@click.option(
    "-r",
    "--grapl-region",
    type=click.Choice(docker_swarm_ops.REGION_TO_AMI_ID.keys()),
    envvar="GRAPL_REGION",
    help="grapl region to target [$GRAPL_REGION]",
    required=True,
)
@click.option(
    "-n",
    "--grapl-deployment-name",
    type=click.STRING,
    envvar="GRAPL_DEPLOYMENT_NAME",
    help="grapl deployment name [$GRAPL_DEPLOYMENT_NAME]",
    required=True,
)
@click.option(
    "-g",
    "--grapl-version",
    type=click.STRING,
    envvar="GRAPL_VERSION",
    help="grapl version [$GRAPL_VERSION]",
    required=True,
)
@click.option(
    "-p",
    "--aws-profile",
    type=click.STRING,
    envvar="AWS_PROFILE",
    help="aws auth profile [$AWS_PROFILE]",
    default="default",
)
@click.pass_context
def main(
    ctx: click.Context,
    grapl_region: str,
    grapl_deployment_name: str,
    grapl_version: str,
    aws_profile: str,
) -> None:
    session = boto3.Session(profile_name=aws_profile)
    ctx.obj = GraplctlState(
        grapl_region,
        grapl_deployment_name,
        grapl_version,
        aws_profile,
        ec2=session.resource("ec2", region_name=grapl_region),
        ssm=session.client("ssm", region_name=grapl_region),
        cloudwatch=session.client("cloudwatch", region_name=grapl_region),
        sns=session.client("sns", region_name=grapl_region),
        route53=session.client("route53", region_name=grapl_region),
    )


#
# aws deployment & provisioning commands
#


@main.group(help="commands for managing grapl aws resources", name="aws")
def aws():
    pass


@aws.command(help="deploy grapl to aws", name="deploy")
@click.option(
    "-a", "--all", is_flag=True, required=True, help="all services and resources"
)
@click.option(
    "-t",
    "--dgraph-instance-type",
    type=click.Choice(choices=("i3.large", "i3.xlarge", "i3.2xlarge")),
    help="ec2 instance type for dgraph swarm nodes",
    required=True,
)
@click.argument(
    "grapl_root",
    type=click.Path(exists=True, file_okay=False, resolve_path=True),
    required=True,
)
@click.pass_obj
def aws_deploy(
    graplctl_state: GraplctlState, all: bool, dgraph_instance_type: str, grapl_root: str
):
    click.echo("deploying grapl cdk stacks to aws")
    aws_cdk_ops.deploy_grapl(
        grapl_root=pathlib.Path(grapl_root).absolute(),
        aws_profile=graplctl_state.aws_profile,
        stdout=click.get_binary_stream("stdout"),
        stderr=click.get_binary_stream("stderr"),
    )
    click.echo("deployed grapl cdk stacks to aws")

    click.echo("creating dgraph cluster in aws")
    _create_dgraph(graplctl_state=graplctl_state, instance_type=dgraph_instance_type)
    click.echo("created dgraph cluster in aws")


@aws.command(help="tear down grapl in aws", name="destroy")
@click.option(
    "-a", "--all", is_flag=True, required=True, help="all services and resources"
)
@click.argument(
    "grapl_root",
    type=click.Path(exists=True, file_okay=False, resolve_path=True),
    required=True,
)
@click.pass_obj
def aws_destroy(graplctl_state: GraplctlState, all: bool, grapl_root: str):
    click.echo("destroying all grapl aws resources")

    for swarm_id in docker_swarm_ops.swarm_ids(
        ec2=graplctl_state.ec2,
        deployment_name=graplctl_state.grapl_deployment_name,
        version=graplctl_state.grapl_version,
        region=graplctl_state.grapl_region,
    ):
        click.echo(f"removing swarm dns records for swarm {swarm_id}")
        _remove_dgraph_dns(graplctl_state=graplctl_state, swarm_id=swarm_id)
        click.echo(f"removed swarm dns records for swarm {swarm_id}")

        click.echo(f"destroying swarm cluster {swarm_id}")
        _destroy_swarm(graplctl_state=graplctl_state, swarm_id=swarm_id)
        click.echo(f"destroyed swarm cluster {swarm_id}")

    click.echo("destroying grapl cdk aws stacks")
    aws_cdk_ops.destroy_grapl(
        grapl_root=pathlib.Path(grapl_root).absolute(),
        aws_profile=graplctl_state.aws_profile,
        stdout=click.get_binary_stream("stdout"),
        stderr=click.get_binary_stream("stderr"),
    )
    click.echo("destroyed grapl cdk aws stacks")

    click.echo("destroyed all grapl aws resources")


@aws.command(help="provision the grapl deployment", name="provision")
@click.pass_obj
def aws_provision(graplctl_state: GraplctlState):
    pass  # FIXME

main.add_command(queues)

#
# swarm operational commands
#


@main.group(help="commands for operating docker swarm clusters", name="swarm")
def swarm():
    pass


def _create_swarm(
    graplctl_state: GraplctlState,
    num_managers: int,
    num_workers: int,
    instance_type: str,
    swarm_id: str,
    docker_daemon_config: Optional[Dict] = None,
    extra_init: Optional[Callable[[SSMClient, str, List[Ec2Instance]], None]] = None,
):
    ami_id = docker_swarm_ops.REGION_TO_AMI_ID[graplctl_state.grapl_region.lower()]
    security_group_id = docker_swarm_ops.swarm_security_group_id(
        ec2=graplctl_state.ec2,
        deployment_name=graplctl_state.grapl_deployment_name,
    )
    vpc_id = docker_swarm_ops.swarm_vpc_id(
        ec2=graplctl_state.ec2, swarm_security_group_id=security_group_id
    )

    click.echo(f"retrieving subnet ids in vpc {vpc_id}")
    subnet_ids = set(
        docker_swarm_ops.subnet_ids(
            ec2=graplctl_state.ec2,
            swarm_vpc_id=vpc_id,
            deployment_name=graplctl_state.grapl_deployment_name,
        )
    )
    click.echo(f"retrieved subnet ids in vpc {vpc_id}")

    click.echo(f"creating manager instances in vpc {vpc_id}")
    manager_instances = docker_swarm_ops.create_instances(
        ec2=graplctl_state.ec2,
        ssm=graplctl_state.ssm,
        deployment_name=graplctl_state.grapl_deployment_name,
        region=graplctl_state.grapl_region,
        version=graplctl_state.grapl_version,
        swarm_manager=True,
        swarm_id=swarm_id,
        ami_id=ami_id,
        count=num_managers,
        instance_type=instance_type,
        security_group_id=security_group_id,
        subnet_ids=subnet_ids,
    )
    manager_instance_ids_str = ",".join(w.instance_id for w in manager_instances)
    click.echo(f"created manager instances {manager_instance_ids_str} in vpc {vpc_id}")

    click.echo(f"creating worker instances in vpc {vpc_id}")
    worker_instances = docker_swarm_ops.create_instances(
        ec2=graplctl_state.ec2,
        ssm=graplctl_state.ssm,
        deployment_name=graplctl_state.grapl_deployment_name,
        region=graplctl_state.grapl_region,
        version=graplctl_state.grapl_version,
        swarm_manager=False,
        swarm_id=swarm_id,
        ami_id=ami_id,
        count=num_workers,
        instance_type=instance_type,
        security_group_id=security_group_id,
        subnet_ids=subnet_ids,
    )
    worker_instance_ids_str = ",".join(w.instance_id for w in worker_instances)
    click.echo(f"created worker instances {worker_instance_ids_str} in vpc {vpc_id}")

    all_instances = manager_instances + worker_instances
    instance_ids_str = ",".join(i.instance_id for i in all_instances)

    click.echo(f"initializing instances {instance_ids_str}")
    docker_swarm_ops.init_instances(
        ssm=graplctl_state.ssm,
        deployment_name=graplctl_state.grapl_deployment_name,
        instances=all_instances,
    )
    click.echo(f"initialized instances {instance_ids_str}")

    if extra_init is not None:
        click.echo(f"performing extra initialization on instances {instance_ids_str}")
        extra_init(
            graplctl_state.ssm,
            graplctl_state.grapl_deployment_name,
            all_instances,
        )
        click.echo(f"performed extra initialization on instances {instance_ids_str}")

    if docker_daemon_config is not None:
        click.echo(f"configuring docker daemon on instances {instance_ids_str}")
        docker_swarm_ops.configure_docker_daemon(
            ssm=graplctl_state.ssm,
            deployment_name=graplctl_state.grapl_deployment_name,
            instances=all_instances,
            config=docker_daemon_config,
        )
        click.echo(f"configured docker daemon on instances {instance_ids_str}")

    click.echo(f"restarting daemons on instances {instance_ids_str}")
    docker_swarm_ops.restart_daemons(
        ssm=graplctl_state.ssm,
        deployment_name=graplctl_state.grapl_deployment_name,
        instances=all_instances,
    )
    click.echo(f"restarted daemons on instances {instance_ids_str}")

    manager_instance = manager_instances[0]
    click.echo(
        f"configuring docker swarm cluster manager {manager_instance.instance_id}"
    )
    docker_swarm_ops.init_docker_swarm(
        ec2=graplctl_state.ec2,
        ssm=graplctl_state.ssm,
        deployment_name=graplctl_state.grapl_deployment_name,
        manager_instance=manager_instance,
    )
    click.echo(
        f"configured docker swarm cluster manager {manager_instance.instance_id}"
    )

    if len(manager_instances) > 1:
        click.echo(
            f"extracting docker swarm manager join token from manager {manager_instance.instance_id}"
        )
        manager_join_token = docker_swarm_ops.extract_join_token(
            ssm=graplctl_state.ssm,
            deployment_name=graplctl_state.grapl_deployment_name,
            manager_instance=manager_instance,
            manager=True,
        )
        click.echo(
            f"extracted docker swarm manager join token from manager {manager_instance.instance_id}"
        )

        remaining_manager_instance_ids_str = ",".join(
            w.instance_id for w in manager_instances[1:]
        )
        click.echo(
            f"joining docker swarm manager instances {remaining_manager_instance_ids_str}"
        )
        docker_swarm_ops.join_swarm_nodes(
            ec2=graplctl_state.ec2,
            ssm=graplctl_state.ssm,
            deployment_name=graplctl_state.grapl_deployment_name,
            instances=manager_instances[1:],
            join_token=manager_join_token,
            manager=True,
            manager_ip=manager_instance.private_ip_address,
        )
        click.echo(
            f"joined docker swarm manager instances {remaining_manager_instance_ids_str}"
        )

    click.echo(
        f"extracting docker swarm worker join token from manager {manager_instance.instance_id}"
    )
    worker_join_token = docker_swarm_ops.extract_join_token(
        ssm=graplctl_state.ssm,
        deployment_name=graplctl_state.grapl_deployment_name,
        manager_instance=manager_instance,
        manager=False,
    )
    click.echo(
        f"extracted docker swarm worker join token from manager {manager_instance.instance_id}"
    )

    click.echo(f"joining docker swarm worker instances {worker_instance_ids_str}")
    docker_swarm_ops.join_swarm_nodes(
        ec2=graplctl_state.ec2,
        ssm=graplctl_state.ssm,
        deployment_name=graplctl_state.grapl_deployment_name,
        instances=worker_instances,
        join_token=worker_join_token,
        manager=False,
        manager_ip=manager_instance.private_ip_address,
    )
    click.echo(f"joined docker swarm worker instances {worker_instance_ids_str}")


@swarm.command(
    help="start ec2 instances and join them as a docker swarm cluster",
    name="create",
)
@click.option(
    "-m",
    "--num-managers",
    type=click.IntRange(min=1, max=None),
    help="number of manager nodes to create",
    required=True,
)
@click.option(
    "-w",
    "--num-workers",
    type=click.IntRange(min=1, max=None),
    help="number of worker nodes to create",
    required=True,
)
@click.option(
    "-t",
    "--instance-type",
    type=click.STRING,
    help="EC2 instance type for swarm nodes",
    required=True,
)
@click.option(
    "-i",
    "--swarm-id",
    type=click.STRING,
    help="unique id for this swarm cluster (random default)",
    default=str(uuid.uuid4()),
)
@pass_graplctl_state
def swarm_create(
    graplctl_state: GraplctlState,
    num_managers: int,
    num_workers: int,
    instance_type: str,
    swarm_id: str,
) -> None:
    _create_swarm(
        graplctl_state=graplctl_state,
        num_managers=num_managers,
        num_workers=num_workers,
        instance_type=instance_type,
        swarm_id=swarm_id,
    )


@swarm.command(help="list swarm ids for each of the swarm clusters", name="ls")
@pass_graplctl_state
def swarm_ls(graplctl_state: GraplctlState):
    for swarm_id in docker_swarm_ops.swarm_ids(
        ec2=graplctl_state.ec2,
        deployment_name=graplctl_state.grapl_deployment_name,
        region=graplctl_state.grapl_region,
        version=graplctl_state.grapl_version,
    ):
        click.echo(swarm_id)


@swarm.command(help="get instance ids for a docker swarm's managers", name="managers")
@click.option(
    "-i",
    "--swarm-id",
    type=click.STRING,
    help="unique id of the swarm cluster",
    required=True,
)
@pass_graplctl_state
def swarm_managers(graplctl_state: GraplctlState, swarm_id: str):
    for manager_instance in docker_swarm_ops.swarm_instances(
        ec2=graplctl_state.ec2,
        deployment_name=graplctl_state.grapl_deployment_name,
        version=graplctl_state.grapl_version,
        region=graplctl_state.grapl_region,
        swarm_id=swarm_id,
        swarm_manager=True,
    ):
        click.echo(manager_instance.instance_id)


def _destroy_swarm(graplctl_state: GraplctlState, swarm_id: str):
    for instance in docker_swarm_ops.swarm_instances(
        ec2=graplctl_state.ec2,
        deployment_name=graplctl_state.grapl_deployment_name,
        version=graplctl_state.grapl_version,
        region=graplctl_state.grapl_region,
        swarm_id=swarm_id,
    ):
        graplctl_state.ec2.Instance(instance.instance_id).terminate(
            InstanceIds=[instance.instance_id]
        )
        click.echo(f"terminated instance {instance.instance_id}")


@swarm.command(help="terminate a docker swarm cluster's instances", name="destroy")
@click.option(
    "-i",
    "--swarm-id",
    type=click.STRING,
    help="unique id of the swarm cluster",
    required=True,
)
@click.confirmation_option(prompt="are you sure you want to destroy the swarm cluster?")
@pass_graplctl_state
def swarm_destroy(graplctl_state: GraplctlState, swarm_id: str):
    click.echo(f"destroying swarm {swarm_id}")
    _destroy_swarm(graplctl_state=graplctl_state, swarm_id=swarm_id)
    click.echo(f"destroyed swarm {swarm_id}")


@swarm.command(help="execute a command on a swarm manager", name="exec")
@click.option(
    "-i",
    "--swarm-id",
    type=click.STRING,
    help="unique id of the swarm cluster",
    required=True,
)
@click.argument("command", nargs=-1, type=click.STRING)
@pass_graplctl_state
def swarm_exec(graplctl_state: GraplctlState, swarm_id: str, command: List[str]):
    click.echo(
        docker_swarm_ops.exec_(
            ec2=graplctl_state.ec2,
            ssm=graplctl_state.ssm,
            deployment_name=graplctl_state.grapl_deployment_name,
            region=graplctl_state.grapl_region,
            version=graplctl_state.grapl_version,
            swarm_id=swarm_id,
            command=command,
        )
    )


@swarm.command(help="scale up a docker swarm cluster", name="scale")
@click.option(
    "-m",
    "--num-managers",
    type=click.IntRange(min=0, max=None),
    help="number of additional manager nodes to create",
    default=0,
)
@click.option(
    "-w",
    "--num-workers",
    type=click.IntRange(min=0, max=None),
    help="number of additional worker nodes to create",
    default=0,
)
@click.option(
    "-t",
    "--instance-type",
    type=click.STRING,
    help="EC2 instance type for swarm nodes",
    required=True,
)
@click.option(
    "-i",
    "--swarm-id",
    type=click.STRING,
    help="unique id of the swarm cluster",
    required=True,
)
@pass_graplctl_state
def swarm_scale(
    graplctl_state: GraplctlState,
    num_managers: int,
    num_workers: int,
    instance_type: str,
    swarm_id: str,
):
    if num_managers + num_workers < 1:
        raise click.BadOptionUsage(
            option_name="--num-managers|--num-workers",
            message="must specify nonzero number of managers and/or workers",
        )

    security_group_id = docker_swarm_ops.swarm_security_group_id(
        ec2=graplctl_state.ec2,
        deployment_name=graplctl_state.grapl_deployment_name,
    )
    vpc_id = docker_swarm_ops.swarm_vpc_id(
        ec2=graplctl_state.ec2, swarm_security_group_id=security_group_id
    )
    subnet_ids = set(
        docker_swarm_ops.subnet_ids(
            ec2=graplctl_state.ec2,
            swarm_vpc_id=vpc_id,
            deployment_name=graplctl_state.grapl_deployment_name,
        )
    )
    manager_instance = next(
        docker_swarm_ops.swarm_instances(
            ec2=graplctl_state.ec2,
            deployment_name=graplctl_state.grapl_deployment_name,
            version=graplctl_state.grapl_version,
            region=graplctl_state.grapl_region,
            swarm_id=swarm_id,
            swarm_manager=True,
        )
    )

    manager_instances = []
    if num_managers > 0:
        click.echo(f"creating manager instances in vpc {vpc_id}")
        manager_instances.extend(
            docker_swarm_ops.create_instances(
                ec2=graplctl_state.ec2,
                ssm=graplctl_state.ssm,
                deployment_name=graplctl_state.grapl_deployment_name,
                region=graplctl_state.grapl_region,
                version=graplctl_state.grapl_version,
                swarm_manager=True,
                swarm_id=swarm_id,
                ami_id=docker_swarm_ops.REGION_TO_AMI_ID[graplctl_state.grapl_region],
                count=num_managers,
                instance_type=instance_type,
                security_group_id=security_group_id,
                subnet_ids=subnet_ids,
            )
        )
        click.echo(
            f"created manager instances {','.join(i.instance_id for i in manager_instances)} in vpc {vpc_id}"
        )

    worker_instances = []
    if num_workers > 0:
        click.echo(f"creating worker instances in vpc {vpc_id}")
        worker_instances = docker_swarm_ops.create_instances(
            ec2=graplctl_state.ec2,
            ssm=graplctl_state.ssm,
            deployment_name=graplctl_state.grapl_deployment_name,
            region=graplctl_state.grapl_region,
            version=graplctl_state.grapl_version,
            swarm_manager=False,
            swarm_id=swarm_id,
            ami_id=docker_swarm_ops.REGION_TO_AMI_ID[graplctl_state.grapl_region],
            count=num_managers,
            instance_type=instance_type,
            security_group_id=security_group_id,
            subnet_ids=subnet_ids,
        )
        click.echo(
            f"created worker instances {','.join(i.instance_id for i in worker_instances)} in vpc {vpc_id}"
        )

    all_instances = manager_instances + worker_instances
    click.echo(
        f"initializing instances {','.join(i.instance_id for i in all_instances)}"
    )
    docker_swarm_ops.init_instances(
        ssm=graplctl_state.ssm,
        deployment_name=graplctl_state.grapl_deployment_name,
        instances=all_instances,
    )
    click.echo(
        f"initialized instances {','.join(i.instance_id for i in all_instances)}"
    )

    if len(manager_instances) > 0:
        click.echo(
            f"extracting docker swarm manager join token from manager {manager_instance.instance_id}"
        )
        manager_join_token = docker_swarm_ops.extract_join_token(
            ssm=graplctl_state.ssm,
            deployment_name=graplctl_state.grapl_deployment_name,
            manager_instance=manager_instance,
            manager=True,
        )
        click.echo(
            f"extracted docker swarm manager join token from manager {manager_instance.instance_id}"
        )

        click.echo(
            f"joining docker swarm manager instances {','.join(i.instance_id for i in manager_instances)}"
        )
        docker_swarm_ops.join_swarm_nodes(
            ec2=graplctl_state.ec2,
            ssm=graplctl_state.ssm,
            deployment_name=graplctl_state.grapl_deployment_name,
            instances=manager_instances,
            join_token=manager_join_token,
            manager=True,
            manager_ip=manager_instance.private_ip_address,
        )
        click.echo(
            f"joined docker swarm manager instances {','.join(i.instance_id for i in manager_instances)}"
        )

    if len(worker_instances) > 0:
        click.echo(
            f"extracting docker swarm worker join token from manager {manager_instance.instance_id}"
        )
        worker_join_token = docker_swarm_ops.extract_join_token(
            ssm=graplctl_state.ssm,
            deployment_name=graplctl_state.grapl_deployment_name,
            manager_instance=manager_instance,
            manager=False,
        )
        click.echo(
            f"extracted docker swarm worker join token from manager {manager_instance.instance_id}"
        )

        click.echo(
            f"joining docker swarm worker instances {','.join(i.instance_id for i in worker_instances)}"
        )
        docker_swarm_ops.join_swarm_nodes(
            ec2=graplctl_state.ec2,
            ssm=graplctl_state.ssm,
            deployment_name=graplctl_state.grapl_deployment_name,
            instances=worker_instances,
            join_token=worker_join_token,
            manager=False,
            manager_ip=manager_instance.private_ip_address,
        )
        click.echo(
            f"joined docker swarm worker instances {','.join(i.instance_id for i in worker_instances)}"
        )


#
# dgraph operational commands
#


@main.group(help="commands for operating dgraph", name="dgraph")
def dgraph():
    pass


def _create_dgraph(graplctl_state: GraplctlState, instance_type: str) -> None:
    swarm_id = f"{graplctl_state.grapl_deployment_name.lower()}-dgraph-swarm"
    click.echo(f"creating swarm {swarm_id}")
    _create_swarm(
        graplctl_state=graplctl_state,
        num_managers=1,
        num_workers=2,
        instance_type=instance_type,
        swarm_id=swarm_id,
        docker_daemon_config={"data-root": "/dgraph"},
        extra_init=dgraph_ops.init_dgraph,
    )
    click.echo(f"created swarm {swarm_id}")

    manager_instance = next(
        docker_swarm_ops.swarm_instances(
            ec2=graplctl_state.ec2,
            deployment_name=graplctl_state.grapl_deployment_name,
            version=graplctl_state.grapl_version,
            region=graplctl_state.grapl_region,
            swarm_id=swarm_id,
            swarm_manager=True,
        )
    )

    swarm_instances = list(
        docker_swarm_ops.swarm_instances(
            ec2=graplctl_state.ec2,
            deployment_name=graplctl_state.grapl_deployment_name,
            version=graplctl_state.grapl_version,
            region=graplctl_state.grapl_region,
            swarm_id=swarm_id,
        )
    )

    click.echo(f"waiting 5min for cloudwatch metrics to propagate...")
    with click.progressbar(_ticker(300), length=300) as bar:
        for _ in bar:
            continue

    click.echo(f"creating disk usage alarms for dgraph in swarm {swarm_id}")
    for instance in swarm_instances:
        dgraph_ops.create_disk_usage_alarms(
            cloudwatch=graplctl_state.cloudwatch,
            sns=graplctl_state.sns,
            instance_id=instance.instance_id,
            deployment_name=graplctl_state.grapl_deployment_name,
        )
    click.echo(f"created disk usage alarms for dgraph in swarm {swarm_id}")

    click.echo(f"deploying dgraph in swarm {swarm_id}")
    dgraph_ops.deploy_dgraph(
        ssm=graplctl_state.ssm,
        deployment_name=graplctl_state.grapl_deployment_name,
        manager_instance=manager_instance,
        worker_instances=tuple(
            instance
            for instance in swarm_instances
            if Tag(key="grapl-swarm-role", value="swarm-worker") in instance.tags
        ),
    )
    click.echo(f"deployed dgraph in swarm {swarm_id}")

    click.echo(f"updating dns A records for dgraph in swarm {swarm_id}")
    hosted_zone_id = graplctl_state.route53.list_hosted_zones_by_name(
        DNSName=f"{graplctl_state.grapl_deployment_name.lower()}.dgraph.grapl"
    )["HostedZones"][0]["Id"]
    for instance in docker_swarm_ops.swarm_instances(
        ec2=graplctl_state.ec2,
        deployment_name=graplctl_state.grapl_deployment_name,
        version=graplctl_state.grapl_version,
        region=graplctl_state.grapl_region,
        swarm_id=swarm_id,
    ):
        dgraph_ops.insert_dns_ip(
            route53=graplctl_state.route53,
            dns_name=f"{graplctl_state.grapl_deployment_name.lower()}.dgraph.grapl",
            ip_address=instance.private_ip_address,
            hosted_zone_id=hosted_zone_id,
        )
    click.echo(f"updated dns A records for dgraph in swarm {swarm_id}")


@dgraph.command(
    help="spin up a swarm cluster and deploy dgraph on it",
    name="create",
)
@click.option(
    "-t",
    "--instance-type",
    type=click.Choice(choices=("i3.large", "i3.xlarge", "i3.2xlarge")),
    help="EC2 instance type for swarm nodes",
    required=True,
)
@pass_graplctl_state
def create_dgraph(graplctl_state: GraplctlState, instance_type: str):
    click.echo(f"creating dgraph cluster of {instance_type} instances")
    _create_dgraph(graplctl_state=graplctl_state, instance_type=instance_type)
    click.echo(f"created dgraph cluster of {instance_type} instances")


def _remove_dgraph_dns(graplctl_state: GraplctlState, swarm_id: str):
    hosted_zone_id = graplctl_state.route53.list_hosted_zones_by_name(
        DNSName=f"{graplctl_state.grapl_deployment_name.lower()}.dgraph.grapl"
    )["HostedZones"][0]["Id"]
    for instance in docker_swarm_ops.swarm_instances(
        ec2=graplctl_state.ec2,
        deployment_name=graplctl_state.grapl_deployment_name,
        version=graplctl_state.grapl_version,
        region=graplctl_state.grapl_region,
        swarm_id=swarm_id,
    ):
        click.echo(
            f"removing dns records for instance {instance.instance_id} swarm {swarm_id}"
        )
        dgraph_ops.remove_dns_ip(
            route53=graplctl_state.route53,
            dns_name=f"{graplctl_state.grapl_deployment_name.lower()}.dgraph.grapl",
            ip_address=instance.private_ip_address,
            hosted_zone_id=hosted_zone_id,
        )
        click.echo(
            f"removed dns records for instance {instance.instance_id} swarm {swarm_id}"
        )


@dgraph.command(help="remove dgraph dns records", name="remove-dns")
@click.option(
    "-i",
    "--swarm-id",
    type=click.STRING,
    help="unique id of the swarm cluster",
    required=True,
)
@click.confirmation_option(
    prompt="are you sure you want to remove the dgraph dns records?"
)
@pass_graplctl_state
def dgraph_remove_dns(graplctl_state: GraplctlState, swarm_id: str):
    click.echo(f"removing dgraph dns records for swarm {swarm_id}")
    _remove_dgraph_dns(graplctl_state=graplctl_state, swarm_id=swarm_id)
    click.echo(f"removed dgraph dns records for swarm {swarm_id}")
