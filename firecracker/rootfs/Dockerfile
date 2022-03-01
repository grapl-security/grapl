# syntax=docker/dockerfile:1.3-labs

FROM ubuntu:20.04 AS rootfs-build

WORKDIR /home/grapl

RUN --mount=type=cache,target=/var/lib/apt/lists,sharing=locked,id=rootfs-apt <<EOF
    apt-get update
    apt-get install --yes --no-install-recommends \
        fuse \
        fuseext2
EOF

COPY docker_runtime docker_runtime

ENTRYPOINT [ "./docker_runtime/create_rootfs_image.sh" ]