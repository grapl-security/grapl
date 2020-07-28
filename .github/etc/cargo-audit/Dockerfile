FROM rust:alpine AS grapl-cargo-audit
RUN apk add --no-cache musl-dev libc6-compat openssl-dev
RUN RUSTFLAGS="-C target-feature=-crt-static" cargo install cargo-audit
COPY . .
CMD [ "cargo", "audit" ]
