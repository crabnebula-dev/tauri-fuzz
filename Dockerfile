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
RUN apt install -y build-essential gdb git wget clang clang-tools libc++-11-dev libc++abi-11-dev llvm 
# Missing package
RUN apt install -y ninja-build libglib2.0-dev python3-venv

# Install Tauri dependencies
# RUN cargo install tauri-cli
RUN apt install -y libwebkit2gtk-4.0-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

ARG FUZZ_DIR=fuzzy-proto
WORKDIR /$FUZZ_DIR

# Copy fuzzers over
COPY fuzzer fuzzer
COPY mini-app mini-app
COPY README.md README.md

# Cache skeleton with cargo-chef
# RUN cargo install cargo-chef
# WORKDIR /$FUZZ_DIR/fuzzer
# RUN cargo chef cook --release --recipe-path recipe.json
# WORKDIR /$FUZZ_DIR/mini-app/src-tauri
# RUN cargo chef cook --release --recipe-path recipe.json
# WORKDIR /$FUZZ_DIR/tauri RUN cargo chef cook --release --recipe-path recipe.json

# Build mini app
# WORKDIR /$FUZZ_DIR/mini-app/src-tauri
# RUN cargo build --release

# Build fuzzer
WORKDIR /$FUZZ_DIR/fuzzer
RUN cargo build --release

# Add missing include path
RUN echo export C_INCLUDE_PATH=/usr/lib/x86_64-linux-gnu/glib-2.0/include/:/$FUZZ_DIR/fuzzer/target/debug/qemu-libafl-bridge/tcg/i386/ >> /root/.bashrc

WORKDIR /$FUZZ_DIR

ENTRYPOINT [ "/bin/bash" ]
