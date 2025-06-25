# syntax=docker.io/docker/dockerfile:1

# Stage 1: Native RISC-V Builder
# Use a RISC-V container to compile the dApp natively, avoiding cross-compilation issues.
# This approach is based on the successful strategy you provided.
FROM --platform=linux/riscv64 public.ecr.aws/ubuntu/ubuntu:24.04 as builder

# Install build dependencies inside the RISC-V container
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    wget \
    pkg-config \
    libssl-dev

# Install Rust
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH
RUN apt-get install -y curl && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile minimal --default-toolchain stable

# Copy source code and build
WORKDIR /opt/cartesi/dapp
COPY . .
RUN cargo build --release

# Stage 2: Final DApp Image
# Create the final, clean image with the compiled binary and Cartesi tools.
FROM --platform=linux/riscv64 ubuntu:22.04

# Install Cartesi machine tools
ARG MACHINE_EMULATOR_TOOLS_VERSION=0.14.1
ADD https://github.com/cartesi/machine-emulator-tools/releases/download/v${MACHINE_EMULATOR_TOOLS_VERSION}/machine-emulator-tools-v${MACHINE_EMULATOR_TOOLS_VERSION}.deb /
RUN dpkg -i /machine-emulator-tools-v${MACHINE_EMULATOR_TOOLS_VERSION}.deb && rm /machine-emulator-tools-v${MACHINE_EMULATOR_TOOLS_VERSION}.deb

LABEL io.cartesi.rollups.sdk_version=0.9.0
LABEL io.cartesi.rollups.ram_size=128Mi

# Install runtime dependencies and set up user
RUN apt-get update && \
    apt-get install -y --no-install-recommends busybox-static && \
    rm -rf /var/lib/apt/lists/* && \
    useradd --create-home --user-group dapp && \
    mkdir -p /data && \
    chown -R dapp:dapp /data

ENV PATH="/opt/cartesi/bin:/opt/cartesi/dapp:${PATH}"

# Copy the natively compiled dApp from the builder stage
WORKDIR /opt/cartesi/dapp
COPY --from=builder /opt/cartesi/dapp/target/release/dapp .

# Set the entrypoint for the Cartesi machine
ENV ROLLUP_HTTP_SERVER_URL="http://127.0.0.1:5004"
ENTRYPOINT ["rollup-init"]
CMD ["dapp"]
