# syntax=docker/dockerfile:1.3-labs

FROM debian:bullseye-slim AS rootfs-build

RUN --mount=type=cache,target=/var/lib/apt/lists,sharing=locked,id=rootfs-apt <<EOF
    apt update
    apt install --yes \
        fuse=2.9.9-5 \
        fuseext2=0.4-1.2+b1
EOF

COPY docker_runtime docker_runtime

ENTRYPOINT [ "./docker_runtime/create_rootfs_image.sh" ]