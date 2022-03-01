# syntax=docker/dockerfile:1.3-labs

FROM debian:bullseye-slim AS rootfs-build

WORKDIR /home/grapl

RUN --mount=type=cache,target=/var/lib/apt/lists,sharing=locked,id=rootfs-apt <<EOF
    apt-get update
    apt-get install --yes --no-install-recommends \
        fuse=2.9.9-5 \
        fuseext2=0.4-1.2+b1 \
        debootstrap=1.0.123
EOF

COPY docker_runtime docker_runtime

ENTRYPOINT [ "./docker_runtime/create_rootfs_image.sh" ]