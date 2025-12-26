# stolen from https://github.com/fly-apps/hello-rust/blob/main/Dockerfile

FROM rust:latest AS builder

WORKDIR /usr/src/app
COPY . .
# Will build and cache the binary and dependent crates in release mode
RUN --mount=type=cache,target=/usr/local/cargo,from=rust:latest,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    cargo build --release && mv ./target/release/fastdl-hc ./hc

# Runtime image
FROM debian:stable-slim

# Run as "app" user
RUN useradd -ms /bin/bash app

USER app
WORKDIR /app

# Get compiled binaries from builder's cargo install directory
COPY --from=builder /usr/src/app/hc /app/hc

# Run the app
CMD ./hc
