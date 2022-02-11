# syntax=docker/dockerfile:1.3-labs
FROM debian:bullseye-slim AS build-firecracker-vm

RUN --mount=type=cache,target=/var/lib/apt/lists,sharing=locked,id=build-firecracker-apt <<EOF
    apt update
    apt install -y --no-install-recommends \
        build-essential=12.9 \
        ca-certificates=20210119 \
        git=1:2.30.2-1 \
        curl=7.74.0-1.3+deb11u1
EOF

# Install Docker. Inside a Docker container. Yeah.
# The Firecracker tooling, itself, uses Firecracker.
RUN <<EOF
    curl --proto "=https" \
        --tlsv1.2 \
        --silent \
        --show-error \
        --location \
        https://get.docker.com/ | sh
EOF

# Give the internal-docker access to host's docker.
VOLUME ["/var/run/docker.sock"]

# https://github.com/firecracker-microvm/firecracker/blob/main/docs/rootfs-and-kernel-setup.md
RUN <<EOF
git clone https://github.com/firecracker-microvm/firecracker
cd firecracker
config="resources/guest_configs/microvm-kernel-x86_64-4.14.config"
./tools/devtool build_kernel -c $config -n 8
EOF

FROM scratch AS firecracker-vm
COPY --from=build-firecracker-vm /firecracker/kernel .