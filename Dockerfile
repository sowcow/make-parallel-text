FROM ubuntu:22.04 AS base

# -- Install dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    wget \
    unzip \
    ca-certificates

# -- Download libtorch (CPU version) -- (not latest to satisfy other dependencies)
ARG LIBTORCH_URL="https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-2.4.0%2Bcpu.zip"
WORKDIR /tmp
RUN wget --quiet ${LIBTORCH_URL} -O libtorch.zip && \
    unzip libtorch.zip -d /opt && \
    rm libtorch.zip

# Make libtorch discoverable (for C++ or tch-rs)
ENV LIBTORCH=/opt/libtorch
ENV CMAKE_PREFIX_PATH=/opt/libtorch
ENV LD_LIBRARY_PATH=/opt/libtorch/lib:${LD_LIBRARY_PATH}


FROM base AS builder

# -- Install dependencies for building (Rust, etc.) --
RUN apt-get install -y --no-install-recommends \
    build-essential \
    libssl-dev \
    pkg-config \
    curl \
    && rm -rf /var/lib/apt/lists/*

# -- Install Rust via rustup --
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

# Copy and extract only Git-tracked files
COPY rust.tar.gz /app/
RUN tar -xzf rust.tar.gz && rm rust.tar.gz
#&& rm rust/Cargo.lock

# Build the project
RUN cargo build --release


FROM base
WORKDIR /app
COPY --from=builder /app/target/release/make-parallel-text ./make-parallel-text
