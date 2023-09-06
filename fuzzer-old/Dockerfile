# syntax=docker/dockerfile:1.2
FROM ubuntu:23.04 AS fuzz-proto
LABEL "about"="Docker image for fuzzing prototype for Tauri applications"

# install sccache to cache subsequent builds of dependencies
# RUN cargo install sccache
#
# ENV HOME=/root
# ENV SCCACHE_CACHE_SIZE="1G"
# ENV SCCACHE_DIR=$HOME/.cache/sccache
# ENV RUSTC_WRAPPER="/usr/local/cargo/bin/sccache"
# ENV IS_DOCKER="1"
# RUN sh -c 'echo set encoding=utf-8 > /root/.vimrc' \
#     echo "export PS1='"'[LibAFL \h] \w$(__git_ps1) \$ '"'" >> ~/.bashrc && \
#     mkdir ~/.cargo && \
#     echo "[build]\nrustc-wrapper = \"${RUSTC_WRAPPER}\"" >> ~/.cargo/config


RUN apt update 

# Install Rust 
ARG RUST_VERSION=1.70.0
ENV PATH="/root/.cargo/bin:${PATH}"
RUN apt install -y curl
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y 
RUN rustup toolchain install $RUST_VERSION
RUN rustup default $RUST_VERSION

# Install Tauri dependencies
# RUN cargo install tauri-cli
RUN apt install -y libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev


# Install clang common build tools for LibAFL
ARG LLVM_VERSION=16
RUN apt install -y build-essential gdb git wget clang-$LLVM_VERSION clang-tools-$LLVM_VERSION llvm-$LLVM_VERSION

##### AFL++ for code coverage compiler

# Install dependencins for AFL++ (code coverage)
RUN apt install -y build-essential python3-dev automake cmake git flex bison libglib2.0-dev libpixman-1-dev python3-setuptools cargo libgtk-3-dev
RUN apt install -y lld-$LLVM_VERSION llvm-$LLVM_VERSION clang-$LLVM_VERSION # llvm-dev-16 does not exist


# Build AFL++
WORKDIR /root
RUN git clone https://github.com/AFLplusplus/AFLplusplus
WORKDIR /root/AFLplusplus
ENV LLVM_CONFIG=llvm-config-$LLVM_VERSION
RUN make 
RUN make install

##### Build fuzzer + mini-app

# Copy fuzzers over
ARG FUZZ_DIR=fuzzy-proto
WORKDIR /$FUZZ_DIR
COPY fuzzer fuzzer
COPY mini-app mini-app
COPY README.md README.md

WORKDIR /$FUZZ_DIR/fuzzer
# RUN cargo build --release


## Cache skeleton with cargo-chef
# RUN cargo install cargo-chef
# WORKDIR /$FUZZ_DIR/fuzzer
# RUN cargo chef cook --release --recipe-path recipe.json
# WORKDIR /$FUZZ_DIR/mini-app/src-tauri
# RUN cargo chef cook --release --recipe-path recipe.json
# WORKDIR /$FUZZ_DIR/tauri RUN cargo chef cook --release --recipe-path recipe.json

# Build mini app
# WORKDIR /$FUZZ_DIR/mini-app/src-tauri
# RUN cargo build --release

##### Optionally For QEMU build
# apt-get install -y gcc-$(gcc --version|head -n1|sed 's/\..*//'|sed 's/.* //')-plugin-dev libstdc++-$(gcc --version|head -n1|sed 's/\..*//'|sed 's/.* //')-dev
# apt-get install -y ninja-build # for QEMU mode
# RUN echo export C_INCLUDE_PATH=/usr/lib/x86_64-linux-gnu/glib-2.0/include/:/$FUZZ_DIR/fuzzer/target/debug/qemu-libafl-bridge/tcg/i386/ >> /root/.bashrc

##### Neovim + AstroNvim + Bash config
WORKDIR /root
COPY docker-dev_env.sh docker-dev_env.sh
RUN ./docker-dev_env.sh


WORKDIR /$FUZZ_DIR

ENTRYPOINT [ "/bin/bash" ]
