FROM ubuntu:jammy

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    cmake \
    curl \
    git \
    ninja-build \
    openssh-client \
    && apt-get clean

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init --output /tmp/rustup-init \
    && chmod +x ./tmp/rustup-init \
    && /tmp/rustup-init -y \
        --default-toolchain=stable \
        --profile=default \
    && rm /tmp/rustup-init \
    && chmod -R a+w $RUSTUP_HOME $CARGO_HOME \
