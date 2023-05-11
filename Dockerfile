# syntax=docker/dockerfile:1.2
FROM rust:bullseye AS libafl
LABEL "about"="Qemu Fuzzer Project Docker image"

# install sccache to cache subsequent builds of dependencies
RUN cargo install sccache

ENV HOME=/root
ENV SCCACHE_CACHE_SIZE="1G"
ENV SCCACHE_DIR=$HOME/.cache/sccache
ENV RUSTC_WRAPPER="/usr/local/cargo/bin/sccache"
ENV IS_DOCKER="1"
RUN sh -c 'echo set encoding=utf-8 > /root/.vimrc' \
    echo "export PS1='"'[LibAFL \h] \w$(__git_ps1) \$ '"'" >> ~/.bashrc && \
    mkdir ~/.cargo && \
    echo "[build]\nrustc-wrapper = \"${RUSTC_WRAPPER}\"" >> ~/.cargo/config

RUN rustup component add rustfmt clippy

# Install clang 11, common build tools
RUN apt update 
RUN apt install -y build-essential gdb git wget clang clang-tools libc++-11-dev libc++abi-11-dev llvm ninja-build

# Install Tauri dependencies
RUN cargo install tauri-cli
RUN apt install -y libwebkit2gtk-4.0-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

WORKDIR /hackathon

# Copy fuzzers over
COPY fuzzer fuzzer
COPY mini-app mini-app
COPY README.md README.md

# Cache skeleton with cargo-chef
RUN cargo install cargo-chef
RUN cargo chef cook --release --recipe-path fuzzer/recipe.json
RUN cargo chef cook --release --recipe-path mini-app/src-tauri/recipe.json

# Build mini app
WORKDIR /hackathon/mini-app
RUN cargo tauri build

# Build fuzzer
# WORKDIR /hackathon/fuzzer
# RUN cargo build --release

WORKDIR /hackathon

ENTRYPOINT [ "/bin/bash" ]