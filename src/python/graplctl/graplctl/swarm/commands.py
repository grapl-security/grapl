import uuid
from typing import List

import click
import graplctl.swarm.lib as docker_swarm_ops
from graplctl.common import State, pass_graplctl_state
from mypy_boto3_ec2.literals import InstanceTypeType

#
# swarm operational commands
#


@click.group()
@click.pass_context
@pass_graplctl_state
def swarm(
    graplctl_state: State,
    ctx: click.Context,
) -> None:
    """commands for operating docker swarm clusters"""
    pass


@swarm.command()
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
@click.option(
    "-s" "--swarm-config-bucket",
    type=click.STRING,
    help="Name of the S3 bucket with Swarm config files for the cluster",
    required=True,
    envvar="GRAPL_SWARM_CONFIG_BUCKET",
)
@click.confirmation_option(prompt=f"this will incur aws charges, ok?")
@pass_graplctl_state
def create(
    graplctl_state: State,
    num_managers: int,
    num_workers: int,
    instance_type: InstanceTypeType,
    swarm_id: str,
    swarm_config_bucket: str,
) -> None:
    """start ec2 instances and join them as a docker swarm cluster"""
    docker_swarm_ops.create_swarm(
        graplctl_state=graplctl_state,
        num_managers=num_managers,
        num_workers=num_workers,
        instance_type=instance_type,
        swarm_id=swarm_id,
        swarm_config_bucket=swarm_config_bucket,
    )


@swarm.command()
@pass_graplctl_state
def ls(graplctl_state: State) -> None:
    """list swarm ids for each of the swarm clusters"""
    for swarm_id in docker_swarm_ops.swarm_ls(graplctl_state):
        click.echo(swarm_id)


@swarm.command()
@click.option(
    "-i",
    "--swarm-id",
    type=click.STRING,
    help="unique id of the swarm cluster",
    required=True,
)
@pass_graplctl_state
def managers(graplctl_state: State, swarm_id: str) -> None:
    """get instance ids for a docker swarm's managers"""
    for manager_instance in docker_swarm_ops.swarm_instances(
        ec2=graplctl_state.ec2,
        deployment_name=graplctl_state.grapl_deployment_name,
        version=graplctl_state.grapl_version,
        region=graplctl_state.grapl_region,
        swarm_id=swarm_id,
        swarm_manager=True,
    ):
        click.echo(manager_instance.instance_id)


@swarm.command()
@click.option(
    "-i",
    "--swarm-id",
    type=click.STRING,
    help="unique id of the swarm cluster",
    required=True,
)
@click.confirmation_option(prompt="this will destroy the swarm cluster, ok?")
@pass_graplctl_state
def destroy(graplctl_state: State, swarm_id: str) -> None:
    """terminate a docker swarm cluster's instances"""
    click.echo(f"destroying swarm {swarm_id}")
    docker_swarm_ops.destroy_swarm(graplctl_state=graplctl_state, swarm_id=swarm_id)
    click.echo(f"destroyed swarm {swarm_id}")


@swarm.command(name="exec")
@click.option(
    "-i",
    "--swarm-id",
    type=click.STRING,
    help="unique id of the swarm cluster",
    required=True,
)
@click.option(
    "-s" "--swarm-config-bucket",
    type=click.STRING,
    help="Name of the S3 bucket with Swarm config files for the cluster",
    required=True,
    envvar="GRAPL_SWARM_CONFIG_BUCKET",
)
@click.argument("command", nargs=-1, type=click.STRING)
@pass_graplctl_state
def swarm_exec(
    graplctl_state: State, swarm_id: str, swarm_config_bucket: str, command: List[str]
) -> None:
    """execute a command on a swarm manager"""
    click.echo(
        docker_swarm_ops.exec_(
            ec2=graplctl_state.ec2,
            ssm=graplctl_state.ssm,
            deployment_name=graplctl_state.grapl_deployment_name,
            swarm_config_bucket=swarm_config_bucket,
            region=graplctl_state.grapl_region,
            version=graplctl_state.grapl_version,
            swarm_id=swarm_id,
            command=command,
        )
    )


@swarm.command()
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
@click.option(
    "-s" "--swarm-config-bucket",
    type=click.STRING,
    help="Name of the S3 bucket with Swarm config files for the cluster",
    required=True,
    envvar="GRAPL_SWARM_CONFIG_BUCKET",
)
@click.confirmation_option(prompt=f"this will incur aws charges, ok?")
@pass_graplctl_state
def scale(
    graplctl_state: State,
    num_managers: int,
    num_workers: int,
    instance_type: InstanceTypeType,
    swarm_id: str,
    swarm_config_bucket: str,
) -> None:
    """scale up a docker swarm cluster"""
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
        docker_swarm_ops.grapl_subnet_ids(
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
        swarm_config_bucket=swarm_config_bucket,
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
            swarm_config_bucket=swarm_config_bucket,
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
            swarm_config_bucket=swarm_config_bucket,
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
            swarm_config_bucket=swarm_config_bucket,
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
            swarm_config_bucket=swarm_config_bucket,
            instances=worker_instances,
            join_token=worker_join_token,
            manager=False,
            manager_ip=manager_instance.private_ip_address,
        )
        click.echo(
            f"joined docker swarm worker instances {','.join(i.instance_id for i in worker_instances)}"
        )
