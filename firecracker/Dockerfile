# syntax=docker/dockerfile:1.3-labs
FROM debian:bullseye-slim AS build-firecracker-vm

RUN --mount=type=cache,target=/var/lib/apt/lists,sharing=locked,id=build-firecracker-apt <<EOF
    apt update
    apt install -y --no-install-recommends \
        build-essential=12.9 \
        ca-certificates=20210119 \
        git=1:2.30.2-1
EOF

RUN <<EOF
git clone https://github.com/firecracker-microvm/firecracker
cd firecracker
config="resources/guest_configs/microvm-kernel-x86_64-4.14.config"
./tools/devtool build_kernel -c $config -n 8
EOF